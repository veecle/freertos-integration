#[cfg(feature = "unsafe-hooks-assert")]
pub use on_assert::{OnAssertFn, set_on_assert};

#[cfg(feature = "unsafe-hooks-assert")]
mod on_assert {
    use core::ffi::c_ulong;
    use core::sync::atomic::AtomicPtr;
    use core::sync::atomic::Ordering::{Acquire, Release};
    use core::{mem, ptr};

    /// Alias for the `vAssertCalled` function signature.
    // Keeps all uses of the `on_assert` function in sync.
    pub type OnAssertFn = fn(file_name: &str, line: c_ulong);

    /// `vAssertCalled` hook.
    static ON_ASSERT: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

    /// Sets the `vAssertCalled` hook.
    ///
    /// See [configASSERT][config_assert] for more details.
    ///
    /// [config_assert]: https://www.freertos.org/Documentation/02-Kernel/03-Supported-devices/02-Customization#configassert
    pub fn set_on_assert(on_assert_fn: OnAssertFn) {
        ON_ASSERT.store(on_assert_fn as *mut (), Release);
    }

    // SAFETY:
    // We require the user of this crate to promise to use the correct prototype (declared in
    // `Cargo.toml`) to call this unmangled function from any external code when activating the
    // feature.
    #[unsafe(no_mangle)]
    /// # Safety
    ///
    /// `file_name_ptr` must be valid for [`core::ffi::CStr::from_ptr`] safety requirements, or null.
    unsafe extern "C" fn vAssertCalled(
        file_name_ptr: *const core::ffi::c_char,
        line: core::ffi::c_ulong,
    ) {
        let file_name = if file_name_ptr.is_null() {
            "<unknown>"
        } else {
            // SAFETY: We forward the safety requirements to our caller, except nullability which we checked.
            unsafe { core::ffi::CStr::from_ptr(file_name_ptr) }
                .to_str()
                .unwrap()
        };

        let on_assert_fn = ON_ASSERT.load(Acquire);
        if !on_assert_fn.is_null() {
            // SAFETY: If the pointer is non-null, it must be a pointer to a function (set in `Self::set_on_assert`) and
            // we just checked that the pointer is not null.
            let on_assert_fn: OnAssertFn = unsafe { mem::transmute(on_assert_fn) };
            on_assert_fn(file_name, line)
        }

        panic!("FreeRTOS ASSERT: {}:{}", file_name, line);
    }
}
