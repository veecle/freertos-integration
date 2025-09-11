# Veecle FreeRTOS Integration

Wrapper library to use FreeRTOS APIs in Rust.

## Usage

See [examples](https://github.com/veecle/freertos-integration/tree/main/examples) for usage.

## Tests

All tests are written using the FreeRTOS POSIX port.

Tests are only guaranteed to be sound using ports where the following holds true:

- `vTaskEndScheduler` is available and has not further requirements on the caller.
- `taskYIELD` is interrupt-safe.

### Adding new tests

New tests should be added as separate integration tests in [tests](tests).
Each test must be placed in a separate file to ensure one test per binary.
These tests must use `harness = false` and the `common::test` macro instead of the standard `#[test]` macro, see its documentation for more details.

Starting and stopping the FreeRTOS scheduler from multiple tests in parallel leads to interference between the tests.
The FreeRTOS memory allocator also interacts with the scheduler globals so it must not be used in a multi-threaded binary.
Because of that, integration tests are used where each file in the [tests](tests) directory will be compiled as a separate binary.

Every test must include `pub mod common;`.
Marking the module as `pub` avoids Clippy warnings about unused code for common functionality not used by the specific test.
While `pub mod common;` allows access to shared functionality, the main reason is to use the FreeRTOS-allocator as the global allocator.

## Troubleshooting compilation errors

During compilation there may arise several kinds of errors.

### Linker errors

If a program uses FreeRTOS functions that are not enabled in the `FreeRTOSConfig.h` file, then compiling fails with linker errors.
Refer to [the FreeRTOS customization page][FreeRTOS-customization] to learn how to enable specific functions.

### Rust compiler errors

If the program uses bindings from the `veecle-freertos-sys` crate that has been excluded, then compiling fails with Rust errors.
The `veecle-freertos-sys` crate generates the Rust bindings for FreeRTOS based on the `FreeRTOSConfig.h` file.
Make sure everything is properly configured so none of the required bindings gets excluded.

[FreeRTOS-customization]: https://www.freertos.org/Documentation/02-Kernel/03-Supported-devices/02-Customization
