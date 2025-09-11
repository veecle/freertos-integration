#![expect(missing_docs)]

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{AcqRel, Acquire};

use veecle_freertos_integration::{Duration, Timer};

pub mod common;

#[common::apply(common::test)]
fn timers_stop() {
    common::run_freertos_test(|| {
        static CALLBACK_CALLED: AtomicUsize = AtomicUsize::new(0);

        let timer = Timer::periodic(Some(c"timer_stop"), Duration::from_ms(10), |_| {
            CALLBACK_CALLED.fetch_add(1, AcqRel);
        })
        .unwrap();
        timer.handle().start().unwrap();

        assert_eq!(CALLBACK_CALLED.load(Acquire), 0);
        veecle_freertos_sys::bindings::vTaskDelay(
            10 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS(),
        );
        assert_eq!(CALLBACK_CALLED.load(Acquire), 1);
        timer.handle().stop().unwrap();
        veecle_freertos_sys::bindings::vTaskDelay(
            30 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS(),
        );
        assert_eq!(CALLBACK_CALLED.load(Acquire), 1);
    });
}
