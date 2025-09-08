#![expect(missing_docs)]

use std::sync::atomic::{AtomicBool, Ordering};

use freertos_rust::Task;

pub mod common;

#[common::apply(common::test)]
fn task_start() {
    static STARTED: AtomicBool = AtomicBool::new(false);

    Task::new()
        .start(|_| {
            STARTED.store(true, Ordering::Release);

            common::end_scheduler();
        })
        .unwrap();

    freertos_rust::scheduler::start_scheduler();

    assert!(STARTED.load(Ordering::Acquire));
}
