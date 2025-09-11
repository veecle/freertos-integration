#![expect(missing_docs)]

use veecle_freertos_integration::{CurrentTask, Duration, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_notify_no_action() {
    const NOTIFICATION_VALUE: u32 = 42;

    let task = start_task(|task| {
        let notification_value = task.wait_for_notification(0, 0, Duration::zero()).unwrap();
        assert_eq!(notification_value, NOTIFICATION_VALUE);

        let new_notification_value = task
            .wait_for_notification(0, 0, Duration::from_ms(1000))
            .unwrap();
        assert_eq!(new_notification_value, notification_value);

        common::end_scheduler();
    });

    task.set_notification_value(NOTIFICATION_VALUE);

    start_task(move |_| {
        CurrentTask::delay(Duration::from_ms(10));

        task.notify(TaskNotification::NoAction);

        CurrentTask::suspend();
    });

    veecle_freertos_integration::scheduler::start_scheduler();
}
