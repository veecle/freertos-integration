#![expect(missing_docs)]

use futures::FutureExt;
use veecle_freertos_integration::{CurrentTask, Duration, Task, TaskPriority, channel};

pub mod common;

#[common::apply(common::test)]
fn queue_async_messages_waiting() {
    let (mut sender, mut receiver) = channel::<()>(1).expect("queue to be created");

    Task::new()
        .priority(TaskPriority(2))
        .start(move |_| {
            assert_eq!(sender.messages_waiting(), 0);
            sender.send(()).now_or_never().expect("message to be sent");
            assert_eq!(sender.messages_waiting(), 1);

            CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(receiver.receive().now_or_never(), Some(()));
        assert_eq!(receiver.messages_waiting(), 0);
    });
}
