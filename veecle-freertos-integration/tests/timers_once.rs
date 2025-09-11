#![expect(missing_docs)]

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

use veecle_freertos_integration::{Duration, Timer};

pub mod common;

#[common::apply(common::test)]
fn timers_once() {
    common::run_freertos_test(|| {
        static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

        let timer = Timer::once(Some(c"timer_once"), Duration::from_ms(100), |_| {
            CALLBACK_CALLED.store(true, Release);
        })
        .unwrap();
        timer.handle().start().unwrap();

        veecle_freertos_sys::bindings::vTaskDelay(
            150 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS(),
        );

        assert!(CALLBACK_CALLED.load(Acquire));
    });
}
