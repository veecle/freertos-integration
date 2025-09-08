#![expect(missing_docs)]

use freertos_rust::{BlockingToAsyncQueueTaskBuilder, Duration, Queue, Task, TaskPriority};
use futures::FutureExt;

pub mod common;

#[common::apply(common::test)]
fn queue_blocking_to_async() {
    let queue = Queue::new(1).expect("queue to be created");

    let mut blocking_to_async = BlockingToAsyncQueueTaskBuilder::new(c"test", queue.clone(), 1)
        .priority(TaskPriority(2))
        .stack_size(1024)
        .create()
        .unwrap();

    Task::new()
        .priority(TaskPriority(2))
        .start(move |_| {
            assert_eq!(queue.send((), Duration::from_ms(1000)), Ok(()));

            freertos_rust::CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(blocking_to_async.receive().now_or_never(), Some(()));
    })
}
