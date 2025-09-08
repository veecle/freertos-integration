//! # Veecle FreeRTOS Integration
//!
//! Rust wrapper for [FreeRTOS](http://www.freertos.org/).
//!
//! Requires dynamic memory allocation backed by the operating system.
//!
//! Be sure to check the [FreeRTOS documentation](http://www.freertos.org/RTOS.html).

#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(missing_docs)]

extern crate alloc;

mod allocator;
pub mod hooks;
mod isr;
mod queue;
pub mod scheduler;
pub mod task;
mod timers;
mod units;

pub use veecle_freertos_sys::bindings::{
    BaseType_t, QueueHandle_t, TaskHandle_t, TickType_t, TimerHandle_t, UBaseType_t, eNotifyAction,
    vPortGetHeapStats,
};
pub use veecle_freertos_sys::error::FreeRtosError;

pub use crate::allocator::*;
pub use crate::isr::*;
pub use crate::queue::*;
#[doc(inline)]
pub use crate::task::*;
pub use crate::timers::*;
pub use crate::units::Duration;
