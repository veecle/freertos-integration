#![expect(missing_docs)]

use freertos_rust::{Duration, Timer};
use veecle_freertos_sys::error::FreeRtosError;

pub mod common;

#[common::apply(common::test)]
fn timers_zero_duration() {
    common::run_freertos_test(|| {
        // We need to use the match because `fn(TimerHandle)` doesn't implement `Debug`.
        match Timer::once(Some(c"timer_zero_duration"), Duration::from_ms(0), |_| {}) {
            Ok(_) => {
                panic!("zero duration should fail")
            }
            Err(error) => {
                assert_eq!(error, FreeRtosError::ZeroDuration)
            }
        }

        // We need to use the match because `fn(TimerHandle)` doesn't implement `Debug`.
        match Timer::periodic(Some(c"timer_zero_duration"), Duration::from_ms(0), |_| {}) {
            Ok(_) => {
                panic!("zero duration should fail")
            }
            Err(error) => {
                assert_eq!(error, FreeRtosError::ZeroDuration)
            }
        }
    });
}
