# veecle-freertos-sys

This crate creates the FreeRTOS bindings and, if required, builds the FreeRTOS library.

For more information, see the [veecle-freertos-integration README][veecle_freertos_integration_readme].

[veecle_freertos_integration_readme]: ../veecle-freertos-integration/README.md

## Requirements

This project uses `bindgen` to generate FreeRTOS bindings.
Please ensure that you have installed all the requirements mentioned [here](https://rust-lang.github.io/rust-bindgen/requirements.html).

## FreeRTOSConfig.h

The `FreeRTOSConfig.h` file defines what functionality is available in the compiled library.
This information is required at compile-time in the `build.rs` file of the `veecle-freertos-integration` crate to compile the code only for the available functionality.
