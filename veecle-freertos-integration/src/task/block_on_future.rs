use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll};

use crate::{CurrentTask, Duration, Task};

mod waker {
    use core::task::{RawWaker, RawWakerVTable, Waker};

    use veecle_freertos_sys::bindings::TaskHandle_t;

    use crate::{Task, TaskNotification};

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake, drop);

    /// # Safety
    ///
    /// The handle must be a [`TaskHandle_t`] to a task that will never be deleted.
    unsafe fn clone(handle: *const ()) -> RawWaker {
        // The task must be forever valid so we don't need to track ref-counts.
        RawWaker::new(handle, &VTABLE)
    }

    /// # Safety
    ///
    /// The handle must be a [`TaskHandle_t`] to a still valid task.
    unsafe fn wake(handle: *const ()) {
        let handle: TaskHandle_t = handle.cast_mut().cast();
        // SAFETY:
        // The handle is guaranteed to be a `TaskHandle_t` to a still valid task to by this function's requirement.
        let task = unsafe { Task::from_raw_handle(handle) };
        task.notify(TaskNotification::Increment);
    }

    fn drop(_handle: *const ()) {
        // The task must be forever valid so we don't need to track ref-counts.
    }

    /// Create a [`Waker`] that wakes a [`Task`] via [`Task::notify`].
    pub fn new(task: Task) -> Waker {
        let handle: TaskHandle_t = task.raw_handle();
        Task::assert_no_task_deletion();
        // SAFETY: This must guarantee the safety requirements of the functions used in `VTABLE`:
        //
        //  * `Task` is guaranteed to reference a forever valid undeleted task based on above guarantee.
        //  * We know it is a `TaskHandle_t` because we just created it above.
        unsafe { Waker::new(handle.cast(), &VTABLE) }
    }
}

/// Runs a future to completion on the current task and returns its output value.
///
/// # Panics
///
/// If run from outside a [`Task`].
///
/// ```should_panic
/// veecle_freertos_integration::task::block_on_future(async { 2 + 2 });
/// ```
///
/// # Examples
///
/// ```
/// veecle_freertos_integration::Task::new().start(|_| {
///     let result = veecle_freertos_integration::task::block_on_future(async { 2 + 2 });
///     assert_eq!(result, 4);
///     # unsafe { veecle_freertos_sys::bindings::vTaskEndScheduler() };
/// });
/// # veecle_freertos_integration::scheduler::start_scheduler();
/// ```
pub fn block_on_future<T>(future: impl Future<Output = T>) -> T {
    let task = Task::current().expect(
        "Could not find the task of the current execution context. Ensure that the method is called inside a \
         FreeRTOS task.",
    );

    let waker = waker::new(task);
    let mut context = Context::from_waker(&waker);

    let mut future = pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
            break value;
        }
        CurrentTask::take_notification(true, Duration::max());
    }
}
