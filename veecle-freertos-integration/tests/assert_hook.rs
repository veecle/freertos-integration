#![expect(missing_docs)]

use veecle_freertos_integration::Task;

pub mod common;

// `vAssertCalled` is an `extern "C"` function.
// Because Rust cannot unwind panics in `extern "C"` functions, we need to redirect the program flow out of the assert
// hook.

#[common::apply(common::test)]
fn assert_hook() {
    Task::new()
        .start(|_| {
            veecle_freertos_integration::hooks::set_on_assert(|file_name, line| {
                assert!(file_name.contains("/veecle-freertos-sys/macro-shim.h"));
                assert_eq!(line, 33);

                common::end_scheduler();
            });

            // SAFETY: No safety requirements.
            unsafe {
                veecle_freertos_sys::bindings::shim_configASSERT(0);
            }
        })
        .unwrap();

    veecle_freertos_integration::scheduler::start_scheduler();
}
