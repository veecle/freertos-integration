#![expect(missing_docs)]

use veecle_freertos_integration::Task;

pub mod common;

#[common::apply(common::test)]
fn task_current_get_name() {
    Task::new()
        .name(c"foobar")
        .start(|_| {
            let current_task = Task::current().unwrap();
            assert_eq!(current_task.get_name().unwrap(), "foobar");

            common::end_scheduler();
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
}
