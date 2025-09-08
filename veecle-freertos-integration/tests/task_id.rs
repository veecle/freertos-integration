#![expect(missing_docs)]

use freertos_rust::Task;
use veecle_freertos_sys::bindings::UBaseType_t;

pub mod common;

#[common::apply(common::test)]
fn task_id() {
    const TASK_ID: UBaseType_t = 42;

    let task = Task::new()
        .start(|_| unreachable!("we don't start the scheduler"))
        .unwrap();

    // SAFETY: No synchronization is necessary here because we only have a single thread
    // and are not starting the runtime.
    unsafe { task.set_id(TASK_ID) };

    // SAFETY: No synchronization is necessary here because we only have a single thread
    // and are not starting the runtime.
    let set_id = unsafe { task.get_id() };

    assert_eq!(set_id, TASK_ID);
}
