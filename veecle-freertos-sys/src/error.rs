//! Basic FreeRTOS errors.

use core::fmt::Display;

/// Basic error type for the library.
#[expect(missing_docs)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FreeRtosError {
    OutOfMemory,
    QueueSendTimeout,
    QueueReceiveTimeout,
    MutexTimeout,
    Timeout,
    QueueFull,
    StringConversionError,
    TaskNotFound,
    InvalidQueueSize,
    ProcessorHasShutDown,
    ZeroDuration,
}

impl core::error::Error for FreeRtosError {}

impl Display for FreeRtosError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
