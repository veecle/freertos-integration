#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, InterruptContext, Task, TaskPriority, channel};
use futures::FutureExt;

pub mod common;

#[common::apply(common::test)]
fn queue_async_send_from_isr() {
    let (mut sender, mut receiver) = channel::<()>(1).expect("queue to be created");

    Task::new()
        .priority(TaskPriority(2))
        .start(move |_| {
            let mut interrupt_context = InterruptContext::default();
            sender
                .send_from_isr(&mut interrupt_context, ())
                .expect("message to be sent");

            CurrentTask::delay(Duration::infinite());
        })
        .unwrap();

    common::run_freertos_test(move || {
        assert_eq!(receiver.receive().now_or_never(), Some(()));
    });
}
