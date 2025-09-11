#![expect(missing_docs)]

use veecle_freertos_integration::Task;

pub mod common;

#[common::apply(common::test)]
fn task_get_name() {
    let task = Task::new()
        .name(c"foobar")
        .start(|_| unreachable!("we don't start the scheduler"))
        .unwrap();

    assert_eq!(task.get_name().unwrap(), "foobar");
}
