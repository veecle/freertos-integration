use freertos_rust::{FreeRtosAllocator, Task};
pub use macro_rules_attribute::apply;

#[macro_export]
/// An alternative to the `libtest::test` macro that uses `libtest-mimic` to run a single single-threaded test.
///
/// This must be used because the `libtest` framework isn't compatible with the FreeRTOS allocator, even when running a
/// single test it spawns multiple threads.
///
/// The test file must also be added to `Cargo.toml` with:
///
/// ```toml
/// [[test]]
/// name = "<file name>"
/// harness = false
/// ```
///
/// to avoid `libtest` being included.
///
/// Should be applied as `#[common::apply(common::test)]`.
macro_rules! test {
    (fn $name:ident() $body:block) => {
        fn main() {
            libtest_mimic::run(
                &libtest_mimic::Arguments {
                    test_threads: Some(1),
                    ..libtest_mimic::Arguments::from_args()
                },
                vec![libtest_mimic::Trial::test(stringify!($name), || Ok($body))],
            )
            .exit()
        }
    };
}

pub use crate::test;

#[global_allocator]
static GLOBAL: FreeRtosAllocator =
    // SAFETY: The README.md requires one test per-binary using our custom test harness above, which should avoid any
    // multi-threaded interactions with the allocator.
    unsafe { FreeRtosAllocator::new() };

/// Runs `func` within a default-constructed [`Task`].
pub fn run_freertos_test(to_test_fn: impl FnOnce() + Send + 'static) {
    Task::new()
        .start(|_| {
            to_test_fn();

            end_scheduler();
        })
        .unwrap();

    freertos_rust::scheduler::start_scheduler();
}

/// Safe wrapper for [`vTaskEndScheduler`](veecle_freertos_sys::bindings::vTaskEndScheduler) for tests only.
pub fn end_scheduler() {
    // SAFETY: The README.md requires tests to be run using the FreeRTOS POSIX port.
    // On the FreeRTOS POSIX port, `vTaskEndScheduler` does not have any requirements on the caller.
    unsafe {
        veecle_freertos_sys::bindings::vTaskEndScheduler();
    }
}

/// Starts a task in tests without error handling.
pub fn start_task<F>(func: F) -> Task
where
    F: FnOnce(Task),
    F: Send + 'static,
{
    Task::new().start(func).unwrap()
}
