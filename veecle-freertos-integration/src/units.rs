//! Expose time units type and implementation utilities.
use veecle_freertos_sys::bindings::{TickType_t, portMAX_DELAY, portTICK_PERIOD_MS};

/// A FreeRTOS duration, internally represented as ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration {
    ticks: TickType_t,
}

impl Duration {
    /// Creates the longest `Duration` a FreeRTOS operation is allowed to wait.
    pub fn max() -> Self {
        Self::infinite()
    }

    /// Creates a new `Duration` from the specified number of milliseconds.
    ///
    /// Because the duration is internally represented in ticks this may not result in an exact duration.
    pub fn from_ms(milliseconds: TickType_t) -> Self {
        Self::from_ticks(milliseconds / portTICK_PERIOD_MS())
    }

    /// Creates a new `Duration` from the specified number of ticks.
    pub fn from_ticks(ticks: TickType_t) -> Self {
        Self { ticks }
    }

    // TODO: If this really is an "infinite" marker, then `max` returning the same thing seems wrong.
    /// Creates an infinite `Duration`.
    pub fn infinite() -> Self {
        Self::from_ticks(portMAX_DELAY())
    }

    /// Creates a zero-tick `Duration`, for non-blocking calls.
    pub fn zero() -> Self {
        Self::from_ticks(0)
    }

    /// Creates the smallest non-zero `Duration`, one tick.
    pub fn eps() -> Self {
        Self::from_ticks(1)
    }

    /// Returns the number of milliseconds contained in this `Duration`.
    pub fn ms(&self) -> TickType_t {
        self.ticks * portTICK_PERIOD_MS()
    }

    /// Returns the number of ticks contained in this `Duration`.
    pub fn ticks(&self) -> TickType_t {
        self.ticks
    }
}
