# Veecle FreeRTOS Integration

Wrapper to use FreeRTOS APIs in Rust.

This project is based on [FreeRTOS-rust][freertos-rust] but has since been mostly rewritten.

[freertos-rust]: https://github.com/lobaro/FreeRTOS-rust

## How it works

The `veecle-freertos-integration` crate generates bindings to the configured (see [Configuration][configuration]) FreeRTOS source code and, if using the `link-freertos` feature, builds and links the FreeRTOS source.

[configuration]: #configuration

## Supported FreeRTOS version

The currently supported FreeRTOS version is: [`V11.2.0`][freertos_version].

[freertos_version]: https://github.com/FreeRTOS/FreeRTOS-Kernel/releases/tag/V11.1.0

## Usage

### Submodules

This projects uses Git submodules, which are required to run tests and others.
You can run `git submodule update --init` to fetch the submodules.

### Features

- `link-freertos`: Links (and builds, depending on env-vars) the FreeRTOS library.
  When using this crate to build a static library for inclusion in a C project, it can be necessary to disable this feature to only link the FreeRTOS library in the final linking stage in the C project.

### Configuration

Environment variables are used to configure the include and source paths for this crate:

- `FREERTOS_CONFIG_INCLUDE_PATH`: Path to the directory containing the `FreeRTOSConfig.h` file.
- `FREERTOS_KERNEL_INCLUDE_PATH`: Path to the FreeRTOS kernel include directory.
- `FREERTOS_KERNEL_PORTMACRO_INCLUDE_PATH`: Path to the FreeRTOS `portmacro` directory.
- `FREERTOS_HEAP_FILE_PATH`: Path to the FreeRTOS heap implementation file.

- `FREERTOS_ADDITIONAL_INCLUDE_PATHS`: One or more paths to additional include directories used when generating bindings and building the FreeRTOS library.
  Multiple paths are separated following the system's convention for the `PATH` environment variable.
  This typically means `:` for Unix and `;` for Windows.
  See [`std::env::split_paths`][split_paths] for details.
- `FREERTOS_ADDITIONAL_INCLUDE_PATHS_BASE`: If set, all paths in `FREERTOS_ADDITIONAL_INCLUDE_PATHS` interpreted as relative to the set base path.

- `BINDINGS_WRAPPER_PREPEND_EXTENSION_PATH`: Path to a file whose contents will be prepended to the bindings `wrapper.h` file.
  This is useful to add `defines` on which the includes of the wrapper rely on.

#### Using a pre-built FreeRTOS library

These environment variables are only taken into account if the `link-freertos` feature is active.
If (and only if) both variables are set, the crate will not build the FreeRTOS library, but instead use the library defined by `LIB_FREERTOS_NAME` & `LIB_FREERTOS_SEARCH_PATH`.

- `LIB_FREERTOS_NAME`: Name of the FreeRTOS library to link to (without `lib` prefix and file ending).
- `LIB_FREERTOS_SEARCH_PATH`: Directory containing the FreeRTOS library.

[split_paths]: https://doc.rust-lang.org/std/env/fn.split_paths.html

##### bindgen

bindgen uses `libclang`, which might not use the correct include directories by default when cross-compiling.
Set `BINDGEN_EXTRA_CLANG_ARGS_[TARGET]` with `TARGET` being the Rust target being compiled for.
Depending on your system, the dashes (`-`) in the target tuple might not be supported in environment variables.
In that case, replace them with underscores (`_`).
For example: `BINDGEN_EXTRA_CLANG_ARGS_thumbv7em_none_eabihf = -Ipath/to/includes`.
See [bindgen's documentation][bindgen_doc] for more information.

[bindgen_doc]: https://github.com/rust-lang/rust-bindgen?tab=readme-ov-file#environment-variables

#### ISR safety

Most `veecle-freertos-integration` methods are not safe to be called from an ISR/interrupt context.
Only `veecle-freertos-integration` methods explicitly documented as ISR-/interrupt-safe are safe to call in an interrupt handler.
Using non-ISR-/interrupt-safe methods within an interrupt context will lead to undefined behavior.

### Used C compiler

To build the FreeRTOS library, this crate depends on the [cc crate](https://docs.rs/crate/cc).
So the C compiler used can be set by using the `CC` environment variable or otherwise defined by internal defaults.
For the ARM architecture this is the [`arm-none-eabi-gcc`][arm_compiler].

[arm_compiler]: https://developer.arm.com/tools-and-software/open-source-software/developer-tools/gnu-toolchain/gnu-rm/downloads

## Examples

To get started there is an example in [examples](examples).

## License

This project is licensed under the [MIT license](LICENSE).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be licensed as MIT without any additional terms or conditions.
