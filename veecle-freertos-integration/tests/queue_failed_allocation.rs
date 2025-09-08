#![expect(missing_docs)]

use freertos_rust::{FreeRtosError, Queue, UBaseType_t};

pub mod common;

#[common::apply(common::test)]
fn queue_failed_allocation() {
    // Due to a sanity check in FreeRTOS, we cannot use `UBaseType_t::MAX` directly.
    let max_size = UBaseType_t::MAX - 1000;

    // We cannot use `assert_eq` with `Queue` as it doesn't implement `PartialEq`.
    assert_eq!(
        Queue::<u8>::new(max_size).expect_err("should fail on out of memory"),
        FreeRtosError::OutOfMemory
    );
}
