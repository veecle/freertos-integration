#![expect(missing_docs)]

use std::sync::atomic::{AtomicBool, Ordering};

use freertos_rust::{CurrentTask, Duration, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_wait_for_notification() {
    const NOTIFICATION_VALUE: u32 = 42;

    static WAITED: AtomicBool = AtomicBool::new(false);

    let task = start_task(|task| {
        let notification_value = task
            .wait_for_notification(0, 0, Duration::from_ms(1000))
            .unwrap();
        assert_eq!(notification_value, NOTIFICATION_VALUE);

        WAITED.store(true, Ordering::Release);

        common::end_scheduler();
    });

    start_task(move |_| {
        CurrentTask::delay(Duration::from_ms(10));

        assert!(!WAITED.load(Ordering::Acquire));

        task.notify(TaskNotification::SetValue(NOTIFICATION_VALUE));

        CurrentTask::suspend();
    });

    freertos_rust::scheduler::start_scheduler();

    assert!(WAITED.load(Ordering::Acquire));
}
