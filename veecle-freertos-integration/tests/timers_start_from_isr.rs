#![expect(missing_docs)]

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

use freertos_rust::{Duration, InterruptContext, Timer};
use veecle_freertos_sys::bindings::pdTRUE;

pub mod common;

// TODO: call `start_from_isr` from an interrupt.
// https://veecle.atlassian.net/browse/DEV-101
#[common::apply(common::test)]
fn timers_start_from_isr() {
    common::run_freertos_test(|| {
        static CALLBACK_CALLED: AtomicBool = AtomicBool::new(false);

        let timer = Timer::once(Some(c"timer"), Duration::from_ms(100), |_| {
            CALLBACK_CALLED.store(true, Release);
        })
        .unwrap();

        let mut interrupt_context = InterruptContext::new();
        timer
            .handle()
            .start_from_isr(&mut interrupt_context)
            .unwrap();
        assert_eq!(interrupt_context.higher_priority_task_woken(), pdTRUE());
        drop(interrupt_context);

        veecle_freertos_sys::bindings::vTaskDelay(
            150 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS(),
        );

        assert!(CALLBACK_CALLED.load(Acquire));
    });
}
