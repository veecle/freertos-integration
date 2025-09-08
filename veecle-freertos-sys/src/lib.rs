//! FreeRTOS low-level Rust bindings.
#![no_std]

pub mod error;
mod macro_wrappers;
mod safe_wrappers;

#[expect(non_upper_case_globals)]
#[expect(non_camel_case_types)]
#[expect(non_snake_case)]
#[allow(missing_docs)]
#[expect(missing_debug_implementations)]
pub mod bindings {
    pub use crate::macro_wrappers::*;
    pub use crate::safe_wrappers::*;
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
