use alloc::boxed::Box;
use core::ffi::CStr;
use core::marker::PhantomData;
use core::ptr;

use veecle_freertos_sys::bindings::{
    TickType_t, TimerHandle_t, pdFALSE, pdTRUE, pvTimerGetTimerID, shim_xTimerChangePeriod,
    shim_xTimerDelete, shim_xTimerStart, shim_xTimerStartFromISR, shim_xTimerStop, xTimerCreate,
    xTimerPendFunctionCall,
};

use crate::units::Duration;
use crate::{FreeRtosError, InterruptContext};

/// Wraps the reference to a FreeRTOS's timer handle, exposing an API to safely communicate with FreeRTOS
/// and perform actions over the corresponding [Timer].
#[derive(Clone, Copy, Debug)]
pub struct TimerHandle(TimerHandle_t);

impl TimerHandle {
    /// Millis to wait for blocking operations.
    const MS_TIMEOUT: TickType_t = 50;

    /// Start the timer.
    pub fn start(&self) -> Result<(), FreeRtosError> {
        // SAFETY:
        // Our handle is a valid undeleted timer based on the field guarantee.
        if unsafe { shim_xTimerStart(self.as_ptr(), Self::block_time()) } == pdTRUE() {
            Ok(())
        } else {
            Err(FreeRtosError::Timeout)
        }
    }

    /// Start the timer from an interrupt.
    pub fn start_from_isr(&self, context: &mut InterruptContext) -> Result<(), FreeRtosError> {
        // SAFETY:
        // Our handle is a valid undeleted timer based on the field guarantee.
        if unsafe { shim_xTimerStartFromISR(self.as_ptr(), context.get_task_field_mut()) }
            == pdTRUE()
        {
            Ok(())
        } else {
            Err(FreeRtosError::QueueSendTimeout)
        }
    }

    /// Stop the timer.
    pub fn stop(&self) -> Result<(), FreeRtosError> {
        // SAFETY:
        // Our handle is a valid undeleted timer based on the field guarantee.
        if unsafe { shim_xTimerStop(self.as_ptr(), Self::block_time()) } == pdTRUE() {
            Ok(())
        } else {
            Err(FreeRtosError::Timeout)
        }
    }

    /// Change the period of the timer.
    pub fn change_period(&self, new_period: Duration) -> Result<(), FreeRtosError> {
        if new_period.ticks() == 0 {
            return Err(FreeRtosError::ZeroDuration);
        }
        // SAFETY:
        // Our handle is a valid undeleted timer based on the field guarantee. This call is unreachable if `new_period`
        // equals zero.
        if unsafe { shim_xTimerChangePeriod(self.as_ptr(), new_period.ticks(), Self::block_time()) }
            == pdTRUE()
        {
            Ok(())
        } else {
            Err(FreeRtosError::Timeout)
        }
    }

    #[inline]
    fn as_ptr(&self) -> TimerHandle_t {
        self.0
    }

    /// Returns the timer id.
    fn id(&self) -> *mut core::ffi::c_void {
        // SAFETY:
        // Our handle is a valid undeleted timer based on the field guarantee.
        unsafe { pvTimerGetTimerID(self.as_ptr()) }
    }

    /// Helper that returns the block time in ticks.
    #[inline]
    fn block_time() -> TickType_t {
        Duration::from_ms(Self::MS_TIMEOUT).ticks()
    }
}

/// A FreeRTOS software timer.
///
/// Note that all operations on a timer are processed by a FreeRTOS internal task
/// that receives messages in a queue. Every operation has an associated waiting time
/// for that queue to get unblocked.
#[derive(Debug)]
pub struct Timer<F>
where
    F: Fn(TimerHandle) + Send + 'static,
{
    handle: TimerHandle,
    callback: PhantomData<F>,
}

impl<F> Timer<F>
where
    F: Fn(TimerHandle) + Send + 'static,
{
    /// Creates a new [`Timer`] which ticks periodically.
    pub fn periodic(
        name: Option<&'static CStr>,
        period: Duration,
        callback: F,
    ) -> Result<Self, FreeRtosError> {
        Self::spawn(name, period.ticks(), true, callback)
    }

    /// Creates a [`Timer`] that ticks once.
    pub fn once(
        name: Option<&'static CStr>,
        period: Duration,
        callback: F,
    ) -> Result<Self, FreeRtosError> {
        Self::spawn(name, period.ticks(), false, callback)
    }

    /// Returns the [`TimerHandle`] of self.
    pub fn handle(&self) -> TimerHandle {
        self.handle
    }

    /// Detach this timer from Rust's memory management. The timer will still be active and
    /// will consume the memory.
    ///
    /// Can be used for timers that will never be changed and don't need to stay in scope.
    ///
    /// This method is safe because resource leak is safe in Rust.
    pub fn detach(self) {
        core::mem::forget(self);
    }

    /// Tries to create a timer with the given strategy.
    fn spawn(
        name: Option<&'static CStr>,
        period: TickType_t,
        auto_reload: bool,
        callback: F,
    ) -> Result<Self, FreeRtosError> {
        if period == 0 {
            return Err(FreeRtosError::ZeroDuration);
        }

        let callback = Box::into_raw(Box::new(callback));

        let name = name.map_or(ptr::null(), |name| name.as_ptr());

        // SAFETY:
        // The content of `name` is an static `CStr`, ensuring it exists for the whole life of the program.
        // This code is unreachable if period equals zero.
        // The callback whose pointer is used as `pvTimerID` is owned by the timer being constructed, ensuring its
        // correctness for the whole lifetime of this timer.
        let handle = unsafe {
            xTimerCreate(
                name.cast(),
                period,
                if auto_reload { pdTRUE() } else { pdFALSE() },
                callback.cast(),
                Some(Self::callback_bridge),
            )
        };

        if handle.is_null() {
            // SAFETY: Creating the timer failed so we retain ownership of the callback.
            drop(unsafe { Box::from_raw(callback) });
            return Err(FreeRtosError::OutOfMemory);
        }

        Ok(Self {
            handle: TimerHandle(handle),
            callback: PhantomData,
        })
    }

    /// A callback for FreeRTOS's software timers.
    ///
    /// This method behaves like a bridge between the FreeRTOS API and the callback given by the final user.
    extern "C" fn callback_bridge(handle: TimerHandle_t) {
        let handle = TimerHandle(handle);
        // SAFETY:
        // The callback's pointer is obtained during spawn and assigned as the timer's id, therefore the
        // pointer belongs to a valid callback.
        let callback =
            unsafe { handle.id().cast::<F>().as_ref() }.expect("callback should not be null");
        callback(handle);
    }

    /// Drops the timer's callback.
    extern "C" fn drop_callback(timer_id: *mut core::ffi::c_void, _: u32) {
        let callback = timer_id.cast::<F>();
        assert!(!callback.is_null(), "callback pointer is null");

        // SAFETY:
        // The callback's pointer is obtained during spawn and assigned as the timer's id, therefore the
        // pointer belongs to a valid callback, which is ensured to not be null.
        // Also, since the timer has previously been deleted, which is the only task in charge of calling
        // `callback_bridge`, which, ultimately, would call the callback being dropped, it is sure that timer
        // handle will never be called again and attempt to access the callback.
        drop(unsafe { Box::from_raw(callback) });
    }
}

impl<F> Drop for Timer<F>
where
    F: Fn(TimerHandle) + Send + 'static,
{
    fn drop(&mut self) {
        // get timer's id before deleting it.
        let timer_id = self.handle.id();

        // SAFETY:
        // The timer's handle is always initialized during spawn, and it is not possible to create a timer
        // without spawning it.
        let result = unsafe { shim_xTimerDelete(self.handle.as_ptr(), TimerHandle::block_time()) };

        assert_eq!(result, pdTRUE(), "timer deletion has failed");

        // SAFETY:
        // The handle from which the value of `pvParameter1` is obtained is created during the construction of self,
        // ensuring it is still valid during the execution of `drop_callback`.
        let result = unsafe {
            xTimerPendFunctionCall(
                Some(Self::drop_callback),
                timer_id,
                0,
                TimerHandle::block_time(),
            )
        };

        assert_eq!(result, pdTRUE(), "drop callback scheduling has failed");
    }
}
