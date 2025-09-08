use veecle_freertos_sys::bindings::{TickType_t, vTaskStartScheduler, xTaskGetTickCount};

use crate::Duration;

/// Starts the FreeRTOS scheduler.
///
/// This function isn't expected to return unless `vTaskEndScheduler` is called.
pub fn start_scheduler() {
    // SAFETY:
    // TODO(unsound): The caller must ensure this function is called once, or after `vTaskEndScheduler` has been called.
    unsafe {
        vTaskStartScheduler();
    }
}

/// Returns the count of ticks since [start_scheduler] was called.
pub fn get_tick_count() -> TickType_t {
    // SAFETY:
    // No requirements on the caller in non-ISR contexts. The README stipulates non-ISR-safe methods to not be used in
    // ISR contexts.
    unsafe { xTaskGetTickCount() }
}

/// Like [get_tick_count], but returns the time since [start_scheduler] was called as a [Duration].
pub fn get_tick_count_duration() -> Duration {
    Duration::from_ticks(get_tick_count())
}
