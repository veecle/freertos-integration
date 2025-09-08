// TODO: Replace these with `safe fn` external declarations.
// Blocked on:
//  * https://github.com/rust-lang/rust-clippy/issues/13777
//  * https://youtrack.jetbrains.com/issue/RUST-16099/safe-unsafe-not-supported-in-extern-C-blocks
//  * bindgen support for `safe fn`.

#![allow(non_snake_case)]

use crate::bindings::{__pvPortMalloc, __vPortGetHeapStats, __vTaskDelay, HeapStats_t, TickType_t};

/// Wraps [`pvPortMalloc`] in a safe function.
pub fn pvPortMalloc(xWantedSize: usize) -> *mut ::core::ffi::c_void {
    // SAFETY: No requirements on the caller.
    unsafe { __pvPortMalloc(xWantedSize) }
}

/// Wraps [`__vTaskDelay`] in a safe function.
pub fn vTaskDelay(xTicksToDelay: TickType_t) {
    // SAFETY: No requirements on the caller.
    unsafe { __vTaskDelay(xTicksToDelay) }
}

/// Wraps [`__vPortGetHeapStats`] in a safe function.
///
/// In addition, this function tries to only return sensible values. If all the fields of
/// `HeapStats_t` are zero, it returns `None`.
///
/// This can happen if:
///
/// - the heap implementation doesn't expose `vPortGetHeapStats` and the fallback implementation was linked instead.
///
/// - the heap implementation doesn't set the stats in its `vPortGetHeapStats` implementation.
///
/// - the heap is configured without any memory to allocate.
pub fn vPortGetHeapStats() -> Option<HeapStats_t> {
    const NO_HEAP_STATS: HeapStats_t = HeapStats_t {
        xAvailableHeapSpaceInBytes: 0,
        xSizeOfLargestFreeBlockInBytes: 0,
        xSizeOfSmallestFreeBlockInBytes: 0,
        xNumberOfFreeBlocks: 0,
        xMinimumEverFreeBytesRemaining: 0,
        xNumberOfSuccessfulAllocations: 0,
        xNumberOfSuccessfulFrees: 0,
    };

    let mut heap_stats = NO_HEAP_STATS;

    // SAFETY:
    //
    // The pointer needs to be non-null, point to a valid `HeapStats_t` and the pointed to memory needs to be alive for
    // the duration of the call. We ensure this by creating a `HeapStats_t` on the stack and pass a pointer to it.
    unsafe {
        __vPortGetHeapStats(&raw mut heap_stats);
    }

    let HeapStats_t {
        xAvailableHeapSpaceInBytes,
        xSizeOfLargestFreeBlockInBytes,
        xSizeOfSmallestFreeBlockInBytes,
        xNumberOfFreeBlocks,
        xMinimumEverFreeBytesRemaining,
        xNumberOfSuccessfulAllocations,
        xNumberOfSuccessfulFrees,
    } = heap_stats;

    let has_real_values = xAvailableHeapSpaceInBytes != 0
        || xSizeOfLargestFreeBlockInBytes != 0
        || xSizeOfSmallestFreeBlockInBytes != 0
        || xNumberOfFreeBlocks != 0
        || xMinimumEverFreeBytesRemaining != 0
        || xNumberOfSuccessfulAllocations != 0
        || xNumberOfSuccessfulFrees != 0;

    has_real_values.then_some(heap_stats)
}
