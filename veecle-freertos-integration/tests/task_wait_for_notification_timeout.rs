#![expect(missing_docs)]

use veecle_freertos_integration::{Duration, FreeRtosError};

use crate::common::start_task;

pub mod common;

#[common::apply(common::test)]
fn task_wait_for_notification() {
    start_task(|task| {
        let error = task
            .wait_for_notification(0, 0, Duration::from_ms(1))
            .unwrap_err();

        assert_eq!(error, FreeRtosError::Timeout);

        common::end_scheduler();
    });

    veecle_freertos_integration::scheduler::start_scheduler();
}
