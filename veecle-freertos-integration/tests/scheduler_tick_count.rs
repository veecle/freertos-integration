#![expect(missing_docs)]

use freertos_rust::scheduler::get_tick_count;
use veecle_freertos_sys::bindings::vTaskDelay;

pub mod common;

#[common::apply(common::test)]
fn scheduler_tick_count() {
    common::run_freertos_test(|| {
        let ticks = get_tick_count();
        vTaskDelay(10 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS());

        assert!(ticks < get_tick_count());
    });
}
