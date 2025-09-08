#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_notify() {
    const NOTIFICATION_VALUE: u32 = 42;

    let task = start_task(|_| {
        let notification_value = CurrentTask::take_notification(true, Duration::zero());
        assert_eq!(notification_value, NOTIFICATION_VALUE);

        common::end_scheduler();
    });

    task.notify(TaskNotification::SetValue(NOTIFICATION_VALUE));

    freertos_rust::scheduler::start_scheduler();
}
