//! Wrappers around macros that `bindgen` cannot provide as `const`.

#![allow(non_snake_case)]

use crate::bindings::{
    BaseType_t, TickType_t, shim_pdFALSE, shim_pdTRUE, shim_portMAX_DELAY, shim_portTICK_PERIOD_MS,
    shim_taskYIELD,
};

/// Wraps `portTICK_PERIOD_MS` macro in a function.
pub fn portTICK_PERIOD_MS() -> TickType_t {
    // SAFETY: No requirements on the caller.
    unsafe { shim_portTICK_PERIOD_MS() }
}
/// Wraps `portMAX_DELAY` macro in a function.
pub fn portMAX_DELAY() -> TickType_t {
    // SAFETY: No requirements on the caller.
    unsafe { shim_portMAX_DELAY() }
}

/// Wraps `pdFALSE` macro in a function.
pub fn pdFALSE() -> BaseType_t {
    // SAFETY: No requirements on the caller.
    unsafe { shim_pdFALSE() }
}

/// Wraps `pdTRUE` macro in a function.
pub fn pdTRUE() -> BaseType_t {
    // SAFETY: No requirements on the caller.
    unsafe { shim_pdTRUE() }
}

/// Wraps `taskYIELD` macro in a function.
pub fn taskYIELD() {
    // SAFETY: No requirements on the caller.
    unsafe { shim_taskYIELD() }
}
