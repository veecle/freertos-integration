#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration, TaskNotification};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_notify_set_bits() {
    const BITS_A: u32 = 0b01;
    const BITS_B: u32 = 0b10;

    let task = start_task(|_| {
        let notification_value = CurrentTask::take_notification(true, Duration::zero());
        assert_eq!(notification_value, BITS_A | BITS_B);

        common::end_scheduler();
    });

    task.notify(TaskNotification::SetBits(BITS_A));
    task.notify(TaskNotification::SetBits(BITS_B));

    freertos_rust::scheduler::start_scheduler();
}
