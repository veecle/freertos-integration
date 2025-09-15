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

## Regenerating sample bindings

The sample bindings generated for docs.rs are dependent on details like your local glibc version, so these are only tested in CI.
If you run the same distro as CI (Ubuntu 24.04) you can probably just `BLESS=1 cargo test -p veecle-freertos-sys -- --ignored` to update them, otherwise you can use Docker:

```sh
docker run -it --rm --volume $PWD:/workspace --workdir /workspace --env BLESS=1 index.docker.io/ubuntu:24.04 bash -c '
  set -euo pipefail
  apt-get update
  apt-get install -y gcc libclang-dev curl
  (curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --profile minimal -y)
  source ~/.cargo/env
  cargo test -p veecle-freertos-sys --target-dir /target -- --ignored
'
```
