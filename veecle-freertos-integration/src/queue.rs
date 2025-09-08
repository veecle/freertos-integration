use alloc::sync::Arc;
use core::ffi::CStr;
use core::future::poll_fn;
use core::marker::PhantomData;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::task::Poll;

use atomic_waker::AtomicWaker;
use veecle_freertos_sys::bindings::{
    QueueHandle_t, StackType_t, UBaseType_t, pdTRUE, shim_xQueueCreate, shim_xQueueReceive,
    shim_xQueueSendToBack, uxQueueMessagesWaiting, uxQueueSpacesAvailable, vQueueDelete,
};

use crate::isr::InterruptContext;
use crate::units::Duration;
use crate::{FreeRtosError, Task, TaskPriority};

/// A blocking queue with a finite size. For an asynchronous queue, see [`AsyncQueueSender`] and
/// [`AsyncQueueReceiver`].
///
/// The items are owned by the queue and move ownership when sending.
///
/// Dropping a [`Queue`] does *not* destroy the underlying FreeRTOS queue.
///
/// ## Usage in FFIs
///
/// The implementation works with raw memory representations. This means
/// that the type `T` layout must be understandable by the receiver. This
/// is usually the case for types that are `Send` and `Sized` in Rust.
///
/// If communication with "C" is expected, users `must` ensure the types are
/// C-compatible. This can be achieved by annotating them with the `#[repr(C)]`
/// attribute.
#[derive(Debug)]
pub struct Queue<T> {
    handle: QueueHandle_t,
    item_type: PhantomData<T>,
}

// SAFETY: The queue struct only contains a pointer to the FreeRTOS resource so it is always Send.
unsafe impl<T> Send for Queue<T> {}

// SAFETY: The queue struct only contains a pointer to the FreeRTOS resource so it is always Sync.
unsafe impl<T> Sync for Queue<T> {}

impl<T> Unpin for Queue<T> {}

impl<T> Queue<T>
where
    T: Send + Sized + 'static,
{
    /// Creates a new `Queue` with item type `T` via dynamic memory allocation.
    pub fn new(max_size: UBaseType_t) -> Result<Queue<T>, FreeRtosError> {
        let item_size = size_of::<T>();

        // SAFETY:
        // The binding for `shim_xQueueCreate` requires that `configSUPPORT_DYNAMIC_ALLOCATION` is enabled in the
        // FreeRTOS configuration file. Not having the dynamic allocation enabled generates a compilation error.
        // The NULL result from `shim_xQueueCreate` is captured and converted into a Rust error.
        let handle = unsafe { shim_xQueueCreate(max_size, item_size as UBaseType_t) };

        if handle.is_null() {
            return Err(FreeRtosError::OutOfMemory);
        }

        Ok(Queue {
            handle,
            item_type: PhantomData,
        })
    }

    /// Creates a `Queue` from a raw queue handle.
    ///
    /// # Safety
    ///
    /// `handle` must be a valid FreeRTOS regular queue handle (not semaphore or mutex).
    /// The queue item type `T` must match the `handle`'s item type.
    /// The queue handle must stay valid until the `Queue` and all its clones are dropped.
    #[inline]
    pub unsafe fn from_raw_handle(handle: QueueHandle_t) -> Self {
        Self {
            handle,
            item_type: PhantomData,
        }
    }

    /// Returns the raw queue handle, a pointer to the queue.
    #[inline]
    pub fn raw_handle(&self) -> QueueHandle_t {
        self.handle
    }

    /// Sends an item to the end of the queue. Waits for the queue to have empty space for it.
    pub fn send(&self, item: T, max_wait: Duration) -> Result<(), T> {
        let item = ManuallyDrop::new(item);
        // SAFETY:
        // Our handle is always a valid undeleted queue handle.
        // The queue takes ownership of the value pointed to by `pvItemToQueue` on success.
        // To avoid double-dropping, the `item` is wrapped in `ManuallyDrop`.
        if unsafe {
            shim_xQueueSendToBack(self.handle, (&raw const *item).cast(), max_wait.ticks())
        } == pdTRUE()
        {
            Ok(())
        } else {
            Err(ManuallyDrop::into_inner(item))
        }
    }

    /// Sends an item to the end of the queue, from an interrupt.
    pub fn send_from_isr(&self, context: &mut InterruptContext, item: T) -> Result<(), T> {
        let item = ManuallyDrop::new(item);
        // SAFETY:
        // The queue, and therefore its handle, are created during the construction of Self, ensuring the argument
        // `xQueue` is correct. The value pointed by `pvItemToQueue` is owned by the current function, ensuring
        // it exists while `shim_xQueueSendToBackFromISR` is executed.
        // To avoid double-dropping, the `item` is wrapped in `ManuallyDrop`.
        if unsafe {
            veecle_freertos_sys::bindings::shim_xQueueSendToBackFromISR(
                self.handle,
                (&raw const *item).cast(),
                context.get_task_field_mut(),
            )
        } == pdTRUE()
        {
            Ok(())
        } else {
            Err(ManuallyDrop::into_inner(item))
        }
    }

    /// Waits for an item to be available on the queue.
    pub fn receive(&self, max_wait: Duration) -> Result<T, FreeRtosError> {
        let mut buffer = MaybeUninit::<T>::uninit();

        // SAFETY:
        // The queue, and therefore its handle, are created during the construction of Self, ensuring the argument
        // `xQueue` is correct. The buffer is created right before this call, ensuring its pointer to be valid.
        if unsafe { shim_xQueueReceive(self.handle, buffer.as_mut_ptr().cast(), max_wait.ticks()) }
            == pdTRUE()
        {
            // SAFETY:
            // It is ensured by `xQueueReceive` that pdTRUE is returned if, and only if, a value has been copied into
            // the buffer, allowing us to assume it has been initialized.
            Ok(unsafe { buffer.assume_init() })
        } else {
            Err(FreeRtosError::QueueReceiveTimeout)
        }
    }

    /// Returns the number of messages waiting in the queue.
    pub fn messages_waiting(&self) -> UBaseType_t {
        // SAFETY:
        // The queue, and therefore its handle, are created during the construction of Self, ensuring the argument
        // `xQueue` is correct.
        unsafe { uxQueueMessagesWaiting(self.handle) }
    }

    /// Returns the number of spaces available in the queue.
    pub fn spaces_available(&self) -> UBaseType_t {
        // SAFETY:
        // The queue, and therefore its handle, are created during the construction of Self, ensuring the argument
        // `xQueue` is correct.
        unsafe { uxQueueSpacesAvailable(self.handle) }
    }
}

impl<T> Clone for Queue<T> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            item_type: self.item_type,
        }
    }
}

/// An asynchronous queue with a finite size. For a purely blocking queue, see [`Queue`].
///
/// The items are owned by the queue and move ownership when sending.
///
/// ## Usage in FFIs
///
/// The implementation works with raw memory representations. This means
/// that the type `T` layout must be understandable by the receiver. This
/// is usually the case for types that are `Send` and `Sized` in Rust.
///
/// If communication with "C" is expected, users `must` ensure the types are
/// C-compatible. This can be achieved by annotating them with the `#[repr(C)]`
/// attribute.
#[derive(Debug)]
struct AsyncQueue<T> {
    send_waker: AtomicWaker,
    receive_waker: AtomicWaker,
    queue: Queue<T>,
}

impl<T> AsyncQueue<T>
where
    T: Send + Sized + 'static,
{
    /// Creates a new `AsyncQueue` capable of holding `length` items of type `T` via dynamic memory allocation.
    pub fn new(length: UBaseType_t) -> Result<Self, FreeRtosError> {
        Ok(AsyncQueue {
            send_waker: AtomicWaker::default(),
            receive_waker: AtomicWaker::default(),
            queue: Queue::new(length)?,
        })
    }

    /// Returns the number of messages waiting in the queue.
    #[inline]
    pub fn messages_waiting(&self) -> UBaseType_t {
        self.queue.messages_waiting()
    }
}

impl<T> Drop for AsyncQueue<T> {
    fn drop(&mut self) {
        // SAFETY:
        // The queue, and therefore its handle, are created during the construction of Self, ensuring the argument
        // `xQueue` is correct.
        unsafe {
            vQueueDelete(self.queue.handle);
        }
    }
}

/// An asynchronous queue sender. Can be used to send data to an [`AsyncQueueReceiver`]. Use [`channel`] to create.
///
/// For a purely blocking queue, see [`Queue`].
///
/// The items are owned by the queue and move ownership when sending.
#[derive(Debug)]
pub struct AsyncQueueSender<T>(Arc<AsyncQueue<T>>);

impl<T> AsyncQueueSender<T>
where
    T: Send + Sized + 'static,
{
    /// Returns the number of messages waiting in the queue.
    #[inline]
    pub fn messages_waiting(&self) -> UBaseType_t {
        self.0.messages_waiting()
    }

    /// Sends an item to the end of the queue.
    ///
    /// Waits for the queue to have empty space for up to `max_wait`. If `max_wait` is 0 and the queue is full,
    /// this function returns immediately.
    #[inline]
    pub fn send_blocking(&mut self, item: T, max_wait: Duration) -> Result<(), T> {
        let result = self.0.queue.send(item, max_wait);

        if result.is_ok() {
            self.0.receive_waker.wake();
        }

        result
    }

    /// Sends an item to the end of the queue, from an interrupt.
    #[inline]
    pub fn send_from_isr(&mut self, context: &mut InterruptContext, item: T) -> Result<(), T> {
        let result = self.0.queue.send_from_isr(context, item);

        if result.is_ok() {
            self.0.receive_waker.wake();
        }

        result
    }

    /// Resolves when at least one space is available in the queue.
    async fn poll_ready(&mut self) {
        poll_fn(|cx| {
            self.0.send_waker.register(cx.waker());

            let result = self.0.queue.spaces_available();

            if result == 0 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    /// Asynchronous version of [`send_blocking`](Self::send_blocking).
    ///
    /// This function stays pending until the queue has space for the item.
    pub async fn send(&mut self, item: T) {
        self.poll_ready().await;

        // `T` doesn't implement `Debug`, so we cannot `expect()`.
        if self.0.queue.send(item, Duration::zero()).is_err() {
            // `poll_ready` resolving guarantees a free slot in the queue, so `send` will never fail.
            unreachable!("sending failed unexpectedly");
        };

        self.0.receive_waker.wake();
    }
}

/// An asynchronous queue receiver. Can be used to receive data from an [`AsyncQueueSender`]. Use [`channel`] to create.
///
/// For a purely blocking queue, see [`Queue`].
#[derive(Debug)]
pub struct AsyncQueueReceiver<T>(Arc<AsyncQueue<T>>);

impl<T> AsyncQueueReceiver<T>
where
    T: Send + Sized + 'static,
{
    /// Returns the number of messages waiting in the queue.
    #[inline]
    pub fn messages_waiting(&self) -> UBaseType_t {
        self.0.messages_waiting()
    }

    /// Waits for an item to be available on the queue.
    ///
    /// Returns an item if available and an error if no item is available after `max_wait`.
    pub fn receive_blocking(&mut self, max_wait: Duration) -> Result<T, FreeRtosError> {
        let result = self.0.queue.receive(max_wait);

        if result.is_ok() {
            self.0.send_waker.wake();
        }

        result
    }

    /// Asynchronous version of [`receive_blocking`](Self::receive_blocking).
    ///
    /// This function stays pending until the queue has received an item.
    pub async fn receive(&mut self) -> T {
        poll_fn(|cx| {
            let result = self.0.queue.receive(Duration::zero());

            if let Ok(item) = result {
                self.0.send_waker.wake();
                Poll::Ready(item)
            } else {
                self.0.receive_waker.register(cx.waker());
                Poll::Pending
            }
        })
        .await
    }
}

/// Creates a [`AsyncQueueSender`] [`AsyncQueueReceiver`] pair.
pub fn channel<T>(
    max_size: UBaseType_t,
) -> Result<(AsyncQueueSender<T>, AsyncQueueReceiver<T>), FreeRtosError>
where
    T: Send + Sized + 'static,
{
    let queue = Arc::new(AsyncQueue::new(max_size)?);
    let sender = AsyncQueueSender(queue.clone());
    let receiver = AsyncQueueReceiver(queue);

    Ok((sender, receiver))
}

/// Builder for a task that can receive items from a blocking [`Queue`] and send them to an
/// asynchronous queue.
#[derive(Debug)]
pub struct BlockingToAsyncQueueTaskBuilder<T> {
    name: &'static CStr,
    queue: Queue<T>,
    priority: TaskPriority,
    capacity: UBaseType_t,
    stack_size: StackType_t,
}

impl<T> BlockingToAsyncQueueTaskBuilder<T>
where
    T: Send + Sized + 'static,
{
    /// Creates a new queue bridge task builder.
    pub fn new(name: &'static CStr, queue: Queue<T>, capacity: UBaseType_t) -> Self {
        // This value was determined by trial and error and has worked consistently during tests. It is *not*
        // derived from anything and might need to change with future versions of Rust or the crate.
        const BASE_STACK_SIZE: StackType_t = 256;

        // The FreeRTOS task requires memory for two instances of T to handle resending on failure.
        let data_size = size_of::<T>() as StackType_t * 2;

        Self {
            name,
            queue,
            capacity,
            priority: TaskPriority(1),
            stack_size: BASE_STACK_SIZE + data_size,
        }
    }

    /// Sets the priority of the FreeRTOS task.
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the stack size of the FreeRTOS task.
    pub fn stack_size(mut self, stack_size: StackType_t) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Creates the task and returns a receiver to receive items from the blocking queue in an asynchronous manner.
    pub fn create(self) -> Result<AsyncQueueReceiver<T>, FreeRtosError> {
        let (mut sender, receiver) = channel(self.capacity)?;

        Task::new()
            .name(self.name)
            .stack_size(self.stack_size)
            .priority(self.priority)
            .start(move |_| {
                loop {
                    // Any non-zero delay behaves the same because after a timeout it will try again until the operation
                    // succeeds. The longer the delay the better, since we don't want to waste
                    // resources starting the same operation over and over, so we use the maximum
                    // allowed timeout.
                    let duration = Duration::max();

                    if let Ok(mut data) = self.queue.receive(duration) {
                        while let Err(saved_data) = sender.send_blocking(data, duration) {
                            data = saved_data;
                        }
                    }
                }
            })?;

        Ok(receiver)
    }
}

/// Builder for a task that can receive items from an asynchronous queue and send them to a
/// blocking [`Queue`].
#[derive(Debug)]
pub struct AsyncToBlockingQueueTaskBuilder<T> {
    name: &'static CStr,
    queue: Queue<T>,
    priority: TaskPriority,
    capacity: UBaseType_t,
    stack_size: StackType_t,
}

impl<T> AsyncToBlockingQueueTaskBuilder<T>
where
    T: Send + Sized + 'static,
{
    /// Creates a new queue bridge task builder.
    pub fn new(name: &'static CStr, queue: Queue<T>, capacity: UBaseType_t) -> Self {
        // This value was determined by trial and error and has worked consistently during tests. It is *not*
        // derived from anything and might need to change with future versions of Rust or the crate.
        const BASE_STACK_SIZE: StackType_t = 256;

        // The FreeRTOS task requires memory for two instances of T to handle resending on failure.
        let data_size = size_of::<T>() as StackType_t * 2;

        Self {
            name,
            queue,
            priority: TaskPriority(1),
            capacity,
            stack_size: BASE_STACK_SIZE + data_size,
        }
    }

    /// Sets the priority of the FreeRTOS task.
    pub fn priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the stack size of the FreeRTOS task.
    pub fn stack_size(mut self, stack_size: StackType_t) -> Self {
        self.stack_size = stack_size;
        self
    }

    /// Creates the task and returns a sender to send items to the blocking queue in an asynchronous manner.
    pub fn create(self) -> Result<AsyncQueueSender<T>, FreeRtosError> {
        let (sender, mut receiver) = channel(self.capacity)?;

        Task::new()
            .name(self.name)
            .stack_size(self.stack_size)
            .priority(self.priority)
            .start(move |_| {
                loop {
                    // Any non-zero delay behaves the same because after a timeout it will try again until the operation
                    // succeeds. The longer the delay the better, since we don't want to waste
                    // resources starting the same operation over and over, so we use the maximum
                    // allowed timeout.
                    let duration = Duration::max();

                    if let Ok(mut data) = receiver.receive_blocking(duration) {
                        while let Err(saved_data) = self.queue.send(data, duration) {
                            data = saved_data;
                        }
                    }
                }
            })?;

        Ok(sender)
    }
}
