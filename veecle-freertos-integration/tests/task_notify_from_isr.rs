#![expect(missing_docs)]

use veecle_freertos_integration::{CurrentTask, Duration, InterruptContext, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_notify_from_isr() {
    const NOTIFICATION_VALUE: u32 = 42;

    let task = start_task(|task| {
        let notification_value = task
            .wait_for_notification(0, 0, Duration::from_ms(1000))
            .unwrap();
        assert_eq!(notification_value, NOTIFICATION_VALUE);

        common::end_scheduler();
    });

    start_task(move |_| {
        let mut interrupt_context = InterruptContext::new();
        task.notify_from_isr(
            &mut interrupt_context,
            TaskNotification::SetValue(NOTIFICATION_VALUE),
        )
        .unwrap();

        CurrentTask::suspend();
    });

    veecle_freertos_integration::scheduler::start_scheduler();
}
