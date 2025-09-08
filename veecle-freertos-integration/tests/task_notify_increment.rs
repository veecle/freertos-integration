#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_notify_increment() {
    const NOTIFICATION_VALUE: u32 = 42;

    let task = start_task(|_| {
        let notification_value = CurrentTask::take_notification(true, Duration::zero());
        assert_eq!(notification_value, NOTIFICATION_VALUE + 1);

        common::end_scheduler();
    });

    task.set_notification_value(NOTIFICATION_VALUE);
    task.notify(TaskNotification::Increment);

    freertos_rust::scheduler::start_scheduler();
}
