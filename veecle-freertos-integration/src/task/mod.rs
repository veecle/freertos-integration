//! Utilities for working with FreeRTOS tasks.

use alloc::boxed::Box;
use alloc::ffi::CString;
use alloc::string::String;
use core::ffi::CStr;
use core::ptr::null_mut;

use veecle_freertos_sys::bindings::{
    StackType_t, TaskHandle_t, UBaseType_t, eNotifyAction, eNotifyAction_eIncrement,
    eNotifyAction_eNoAction, eNotifyAction_eSetBits, eNotifyAction_eSetValueWithOverwrite,
    eNotifyAction_eSetValueWithoutOverwrite, pdFALSE, pdTRUE, shim_pcTaskGetName,
    shim_ulTaskNotifyTake, shim_xTaskNotify, shim_xTaskNotifyFromISR, shim_xTaskNotifyWait,
    uxTaskGetStackHighWaterMark, uxTaskGetTaskNumber, vTaskDelay, vTaskSetTaskNumber, vTaskSuspend,
    xTaskCreate, xTaskGetCurrentTaskHandle,
};

pub use self::block_on_future::block_on_future;
use crate::units::Duration;
use crate::{FreeRtosError, InterruptContext};

mod block_on_future;

// SAFETY: All task APIs we expose are fine to call from any task/thread because they use internal locking where
// necessary, or they are marked unsafe and it's up to users to provide thread safety on those specific APIs.
unsafe impl Send for Task {}

// SAFETY: All task APIs we expose are fine to call from any task/thread because they use internal locking where
// necessary, or they are marked unsafe and it's up to users to provide thread safety on those specific APIs.
unsafe impl Sync for Task {}

/// Handle for a FreeRTOS task
#[allow(clippy::new_without_default)]
#[derive(Debug, Clone)]
pub struct Task {
    /// # Safety
    ///
    /// This handle refers to a valid undeleted task, this must be guaranteed on construction and can be assumed on
    /// use.
    task_handle: TaskHandle_t,
}

/// Task's execution priority. Low priority numbers denote low priority tasks.
#[derive(Debug, Copy, Clone)]
pub struct TaskPriority(pub UBaseType_t);

/// Notification to be sent to a task.
#[derive(Debug, Copy, Clone)]
pub enum TaskNotification {
    /// Send the event, unblock the task, the task's notification value isn't changed.
    NoAction,
    /// Perform a logical or with the task's notification value.
    SetBits(u32),
    /// Increment the task's notification value by one.
    Increment,
    /// Set the task's notification value to this value.
    OverwriteValue(u32),
    /// Try to set the task's notification value to this value. Succeeds
    /// only if the task has no pending notifications. Otherwise, the
    /// notification call will fail.
    SetValue(u32),
}

impl TaskNotification {
    fn to_freertos(self) -> (u32, eNotifyAction) {
        match self {
            TaskNotification::NoAction => (0, eNotifyAction_eNoAction),
            TaskNotification::SetBits(v) => (v, eNotifyAction_eSetBits),
            TaskNotification::Increment => (0, eNotifyAction_eIncrement),
            TaskNotification::OverwriteValue(v) => (v, eNotifyAction_eSetValueWithOverwrite),
            TaskNotification::SetValue(v) => (v, eNotifyAction_eSetValueWithoutOverwrite),
        }
    }
}

impl TaskPriority {
    fn to_freertos(self) -> UBaseType_t {
        self.0
    }
}

/// Helper for spawning a new task. Instantiate with [`Task::new()`].
///
/// [`Task::new()`]: struct.Task.html#method.new
#[allow(clippy::new_without_default)]
#[derive(Debug)]
pub struct TaskBuilder {
    task_name: CString,
    task_stack_size: StackType_t,
    task_priority: TaskPriority,
}

impl TaskBuilder {
    /// Set the task's name.
    pub fn name(&mut self, name: &CStr) -> &mut Self {
        self.task_name = name.into();
        self
    }

    /// Set the stack size, in words.
    pub fn stack_size(&mut self, stack_size: StackType_t) -> &mut Self {
        self.task_stack_size = stack_size;
        self
    }

    /// Set the task's priority.
    pub fn priority(&mut self, priority: TaskPriority) -> &mut Self {
        self.task_priority = priority;
        self
    }

    /// Start a new task that can't return a value.
    pub fn start<F>(&self, func: F) -> Result<Task, FreeRtosError>
    where
        F: FnOnce(Task),
        F: Send + 'static,
    {
        Task::spawn(
            &self.task_name,
            self.task_stack_size,
            self.task_priority,
            func,
        )
    }
}

impl Task {
    /// Internal check to prove that once we observe a [`Task`] the backing [`TaskHandle_t`] will never be invalidated.
    ///
    /// There is no runtime cost, the assertion runs at compile time (but this should be called as a normal fn to make
    /// the compiler error messages more readable).
    const fn assert_no_task_deletion() {
        // Using a nested inline const ensures there's only one error emitted to users, no matter how many times this
        // function is called.
        const {
            assert!(
                !cfg!(INCLUDE_vTaskDelete),
                "it is not possible to have a safe wrapper around tasks that may be deleted at any time, you must \
                 disable `INCLUDE_vTaskDelete` to use `Task`"
            )
        }
    }

    /// Prepare a builder object for the new task.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> TaskBuilder {
        TaskBuilder {
            task_name: c"rust_task".into(),
            task_stack_size: 1024,
            task_priority: TaskPriority(1),
        }
    }

    /// # Safety
    ///
    /// `handle` must be a valid FreeRTOS task handle.
    #[inline]
    pub unsafe fn from_raw_handle(handle: TaskHandle_t) -> Self {
        Self {
            task_handle: handle,
        }
    }
    #[inline]
    pub fn raw_handle(&self) -> TaskHandle_t {
        self.task_handle
    }

    unsafe fn spawn_inner(
        f: Box<dyn FnOnce(Task)>,
        name: &CStr,
        stack_size: StackType_t,
        priority: TaskPriority,
    ) -> Result<Task, FreeRtosError> {
        let f = Box::new(f);
        let param_ptr = Box::into_raw(f);

        let (success, task_handle) = {
            let mut task_handle = core::ptr::null_mut();

            // SAFETY:
            // The function `thread_start` cannot finish without panicking, and relies on `extern "C"` doing an
            // abort-on-panic, so it will never return to the scheduler. On success, the memory pointed to by
            // `param_ptr` is leaked to ensure the pointer stays valid until the application terminates.
            // `name` points to a valid, null-terminated cstring and outlives the `xTaskCreate` call, which copies the
            // value pointed to.
            let ret = unsafe {
                xTaskCreate(
                    Some(thread_start),
                    name.as_ptr(),
                    stack_size,
                    param_ptr.cast(),
                    priority.to_freertos(),
                    &mut task_handle,
                )
            };

            (ret == pdTRUE(), task_handle)
        };

        if !success {
            // SAFETY:
            // We created `param_ptr` from a valid `Box` earlier in this function, thus `param_ptr` points to valid
            // memory for `Box::from_raw` and `xTaskCreate` failed, so we retain sole ownership.
            drop(unsafe { Box::from_raw(param_ptr) });
            return Err(FreeRtosError::OutOfMemory);
        }

        use core::ffi::c_void;
        extern "C" fn thread_start(main: *mut c_void) {
            // SAFETY:
            // The `main` pointer is the `param_ptr` passed into `xTaskCreate` above, so we know it is a raw pointer for
            // a `Box<dyn FnOnce(Task)>`.
            let task_main_function = unsafe { Box::from_raw(main.cast::<Box<dyn FnOnce(Task)>>()) };

            task_main_function(
                Task::current().expect("in a task, the current task should be available"),
            );

            panic!("Not allowed to quit the task!");
        }

        Ok(Task { task_handle })
    }

    fn spawn<F>(
        name: &CStr,
        stack_size: StackType_t,
        priority: TaskPriority,
        f: F,
    ) -> Result<Task, FreeRtosError>
    where
        F: FnOnce(Task),
        F: Send + 'static,
    {
        // SAFETY:
        // TODO: `Task::spawn_inner` has no safety requirements, it should probably not be `unsafe`.
        unsafe { Task::spawn_inner(Box::new(f), name, stack_size, priority) }
    }

    /// Get the name of the current task.
    #[allow(clippy::result_unit_err)]
    pub fn get_name(&self) -> Result<String, ()> {
        Task::assert_no_task_deletion();
        // SAFETY: Our handle is a valid undeleted task based on above guarantee.
        let name_ptr = unsafe { shim_pcTaskGetName(self.task_handle) };
        // SAFETY: Not entirely documented, but FreeRTOS returns a valid non-null null-terminated C string.
        unsafe { CStr::from_ptr(name_ptr) }
            .to_str()
            .map_err(|_| ())
            .map(String::from)
    }

    /// Try to find the task of the current execution context.
    pub fn current() -> Result<Task, FreeRtosError> {
        // SAFETY:
        // Calling `xTaskGetCurrentTaskHandle` from outside a task is asserted afterwards, returning a Rust error if the
        // retrived handle is null. TODO(unsound): It is unsound to call `xTaskGetCurrentTaskHandle` from a thread
        // outside FreeRTOS.
        let task_handle = unsafe { xTaskGetCurrentTaskHandle() };

        if task_handle.is_null() {
            return Err(FreeRtosError::TaskNotFound);
        }

        Ok(Task { task_handle })
    }

    /// Forcibly set the notification value for this task.
    pub fn set_notification_value(&self, val: u32) {
        self.notify(TaskNotification::OverwriteValue(val))
    }

    /// Notify this task.
    pub fn notify(&self, notification: TaskNotification) {
        let (value, action) = notification.to_freertos();
        Task::assert_no_task_deletion();
        // SAFETY:
        // Our handle is a valid undeleted task based on the field guarantee.
        unsafe { shim_xTaskNotify(self.task_handle, value, action) };
    }

    /// Notify this task from an interrupt.
    pub fn notify_from_isr(
        &self,
        context: &mut InterruptContext,
        notification: TaskNotification,
    ) -> Result<(), FreeRtosError> {
        let (value, action) = notification.to_freertos();

        Task::assert_no_task_deletion();
        // SAFETY:
        // Our handle is a valid undeleted task based on the field guarantee.
        if unsafe {
            shim_xTaskNotifyFromISR(
                self.task_handle,
                value,
                action,
                context.get_task_field_mut(),
            )
        } == pdTRUE()
        {
            Ok(())
        } else {
            Err(FreeRtosError::QueueFull)
        }
    }

    /// Wait for a notification to be posted.
    pub fn wait_for_notification(
        &self,
        clear_bits_enter: u32,
        clear_bits_exit: u32,
        wait_for: Duration,
    ) -> Result<u32, FreeRtosError> {
        let mut val = 0;

        // TODO: This isn't using this task handle, should it be a `CurrentTask` method?
        //
        // SAFETY:
        // A writable pointer to `val` is passed as the `pulNotificationValue` argument, ensuring it is safe to write
        // the notification value in that local variable.
        if unsafe {
            shim_xTaskNotifyWait(
                clear_bits_enter,
                clear_bits_exit,
                &mut val as *mut _,
                wait_for.ticks(),
            )
        } == pdTRUE()
        {
            Ok(val)
        } else {
            Err(FreeRtosError::Timeout)
        }
    }

    /// Get the minimum amount of stack that was ever left on this task.
    pub fn get_stack_high_water_mark(&self) -> UBaseType_t {
        Task::assert_no_task_deletion();
        // SAFETY:
        // A Task cannot be created without spawning it, ensuring the value of `xTask` is correct.
        unsafe { uxTaskGetStackHighWaterMark(self.task_handle) as UBaseType_t }
    }

    /// # Safety
    ///
    /// This function is not thread safe, you must synchronize all usage of it, [`Task::set_id`], and
    /// `vTaskSetTaskNumber` called with the same task handle manually.
    pub unsafe fn get_id(&self) -> UBaseType_t {
        Task::assert_no_task_deletion();
        // SAFETY:
        // Our handle is a valid undeleted task based on the field guarantee.
        unsafe { uxTaskGetTaskNumber(self.task_handle) }
    }

    /// # Safety
    ///
    /// This function is not thread safe, you must synchronize all usage of it, [`Task::get_id`], `uxTaskGetTaskNumber`
    /// and `vTaskSetTaskNumber` called with the same task handle manually.
    pub unsafe fn set_id(&self, value: UBaseType_t) {
        Task::assert_no_task_deletion();
        // SAFETY:
        // Our handle is a valid undeleted task based on the field guarantee.
        unsafe { vTaskSetTaskNumber(self.task_handle, value) };
    }
}

/// Helper methods to be performed on the task that is currently executing.
#[derive(Debug)]
pub struct CurrentTask;

impl CurrentTask {
    /// Delay the execution of the current task.
    pub fn delay(delay: Duration) {
        vTaskDelay(delay.ticks());
    }

    pub fn suspend() {
        // SAFETY:
        // TODO(unsound): The caller must ensure this is called from inside a FreeRTOS task.
        unsafe { vTaskSuspend(null_mut()) }
    }

    /// Take the notification and either clear the notification value or decrement it by one.
    pub fn take_notification(clear: bool, wait_for: Duration) -> u32 {
        let clear = if clear { pdTRUE() } else { pdFALSE() };

        // SAFETY:
        // TODO(unsound): The caller must ensure this is called from inside a FreeRTOS task.
        unsafe { shim_ulTaskNotifyTake(clear, wait_for.ticks()) }
    }

    /// Get the minimum amount of stack that was ever left on the current task.
    pub fn get_stack_high_water_mark() -> UBaseType_t {
        // SAFETY:
        // TODO(unsound): The caller must ensure this is called from inside a FreeRTOS task.
        unsafe { uxTaskGetStackHighWaterMark(null_mut()) }
    }
}
