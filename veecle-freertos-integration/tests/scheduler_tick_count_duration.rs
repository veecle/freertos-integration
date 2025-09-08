#![expect(missing_docs)]

use freertos_rust::scheduler::get_tick_count_duration;
use veecle_freertos_sys::bindings::vTaskDelay;

pub mod common;

#[common::apply(common::test)]
fn scheduler_tick_count_duration() {
    common::run_freertos_test(|| {
        let ms_since_start = get_tick_count_duration();
        vTaskDelay(10 / veecle_freertos_sys::bindings::portTICK_PERIOD_MS());

        assert!(ms_since_start < get_tick_count_duration());
    });
}
