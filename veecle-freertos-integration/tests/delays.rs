#![expect(missing_docs)]

use freertos_rust::{CurrentTask, Duration};

pub mod common;

#[common::apply(common::test)]
fn delays() {
    let started = std::time::Instant::now();
    common::run_freertos_test(|| {
        for _ in 0..3 {
            CurrentTask::delay(Duration::from_ms(100));
        }
    });
    let elapsed = started.elapsed();
    assert!(
        elapsed > std::time::Duration::from_millis(300),
        "Expected elapsed time to be equal to or more than 300 ms, but was: {} ms",
        elapsed.as_millis()
    );
}
