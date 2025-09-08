#![expect(missing_docs)]

use freertos_rust::Task;

pub mod common;

#[common::apply(common::test)]
fn task_raw_get_name() {
    let task = Task::new()
        .name(c"foobar")
        .start(|_| unreachable!("we don't start the scheduler"))
        .unwrap();

    let raw_handle = task.raw_handle();

    // SAFETY: We just created the raw handle and `INCLUDE_vTaskDelete` must be disabled to compile the `task` feature,
    // so we know it's valid.
    let task = unsafe { Task::from_raw_handle(raw_handle) };

    assert_eq!(task.get_name().unwrap(), "foobar");
}
