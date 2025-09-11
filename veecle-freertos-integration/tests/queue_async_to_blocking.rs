#![expect(missing_docs)]

use futures::FutureExt;
use veecle_freertos_integration::{
    AsyncToBlockingQueueTaskBuilder, Duration, Queue, Task, TaskPriority,
};

pub mod common;

#[common::apply(common::test)]
fn queue_async_to_blocking() {
    let queue = Queue::new(1).expect("queue to be created");

    let mut async_to_blocking = AsyncToBlockingQueueTaskBuilder::new(c"test", queue.clone(), 1)
        .priority(TaskPriority(2))
        .stack_size(1024)
        .create()
        .unwrap();

    Task::new()
        .priority(TaskPriority(2))
        .start(move |_| {
            assert_eq!(async_to_blocking.send(()).now_or_never(), Some(()));

            veecle_freertos_integration::CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(queue.receive(Duration::from_ms(1000)), Ok(()));
    })
}
