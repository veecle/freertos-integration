#![expect(missing_docs)]

use freertos_rust::{Duration, Timer};
use veecle_freertos_sys::error::FreeRtosError;

pub mod common;

#[common::apply(common::test)]
fn timers_change_period_zero_duration() {
    common::run_freertos_test(|| {
        let timer = Timer::periodic(
            Some(c"timers_change_period_zero_duration"),
            Duration::from_ms(10),
            |_| {},
        )
        .unwrap();
        timer.handle().start().unwrap();

        assert_eq!(
            timer.handle().change_period(Duration::from_ms(0)),
            Err(FreeRtosError::ZeroDuration)
        );
    })
}
