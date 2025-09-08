#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Task};
use veecle_freertos_sys::bindings::StackType_t;

pub mod common;

#[common::apply(common::test)]
fn task_stack() {
    const STACK_SIZE: StackType_t = 256;

    Task::new()
        .stack_size(STACK_SIZE)
        .start(|task| {
            let stack_high_water_mark = task.get_stack_high_water_mark();

            assert_ne!(stack_high_water_mark, 0);
            assert!(stack_high_water_mark < STACK_SIZE);

            assert_eq!(
                task.get_stack_high_water_mark(),
                CurrentTask::get_stack_high_water_mark()
            );

            common::end_scheduler();
        })
        .unwrap();

    freertos_rust::scheduler::start_scheduler();
}
