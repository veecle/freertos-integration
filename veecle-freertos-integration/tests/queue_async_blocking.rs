#![expect(missing_docs)]

use veecle_freertos_integration::{
    AsyncToBlockingQueueTaskBuilder, BlockingToAsyncQueueTaskBuilder, Duration, Queue, Task,
};

pub mod common;

#[common::apply(common::test)]
fn queue_async() {
    let queue = Queue::new(1).expect("queue to be created");

    let mut receiver = BlockingToAsyncQueueTaskBuilder::new(c"receiver", queue.clone(), 1)
        .create()
        .unwrap();
    let mut sender = AsyncToBlockingQueueTaskBuilder::new(c"sender", queue, 1)
        .create()
        .unwrap();

    Task::new()
        .start(move |_| {
            sender
                .send_blocking((), Duration::from_ms(1000))
                .expect("message to be sent");

            veecle_freertos_integration::CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(receiver.receive_blocking(Duration::from_ms(1000)), Ok(()));
    })
}
