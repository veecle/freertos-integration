#![expect(missing_docs)]

use veecle_freertos_integration::{FreeRtosError, Task};

pub mod common;

#[common::apply(common::test)]
fn task_current_get_name() {
    let error = Task::current().unwrap_err();

    assert_eq!(error, FreeRtosError::TaskNotFound);
}
