use core::alloc::{GlobalAlloc, Layout};
use core::ptr;

use veecle_freertos_sys::bindings::{portBYTE_ALIGNMENT, pvPortMalloc, vPortFree};

/// Use with:
///
/// ```ignore
/// #[global_allocator]
/// static GLOBAL: FreeRtosAllocator = FreeRtosAllocator;
/// ```

#[derive(Debug)]
pub struct FreeRtosAllocator {
    _private: (),
}

#[repr(C)]
struct OriginalPointer(*mut u8);

impl FreeRtosAllocator {
    /// # Safety
    ///
    /// The FreeRTOS allocator is not safe to use with threads spawned outside the FreeRTOS scheduler.
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }
}

// Rust standard library implements the same strategy for Windows:
// https://github.com/rust-lang/rust/blob/master/library/std/src/sys/alloc/windows.rs#L227
// https://github.com/rust-lang/rust/blob/master/library/std/src/sys/alloc/windows.rs#L157

/// This relies on the `pvPortMalloc` macro to return memory that is aligned to `portByteAlignment`.
// SAFETY:
// The given `Layout` is checked to make sure the proper memory address and amount are used for the (de)allocate
// operation. If there is any error during this process, or there is no way to allocate the requested memory,
// `ptr::null_mut()` is returned by default.
unsafe impl GlobalAlloc for FreeRtosAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // If the requested alignment is smaller than the port alignment, the alignment for the request is fulfilled.
        // This is because every smaller power of two is correctly aligned on every larger power of two.
        // E.g.: requested 8, received 32 => still correctly aligned
        if layout.align() <= usize::from(portBYTE_ALIGNMENT) {
            pvPortMalloc(layout.size()).cast()
        } else {
            // There are architectures where `portBYTE_ALIGNMENT` is smaller than the size of a pointer.
            // Example: https://github.com/FreeRTOS/FreeRTOS-Kernel/blob/main/portable/IAR/AVR_Mega0/portmacro.h#L48-L89
            // Therefore we cannot assume there is enough space for `OriginalPointer` if we only account for alignment
            // of the layout. To ensure that there is enough space and that we can align the
            // `OriginalPointer` as required, we allocate space for both.

            let alloc_information_size =
                size_of::<OriginalPointer>() + align_of::<OriginalPointer>();
            let required_size = layout.align() + layout.size() + alloc_information_size;

            // The memory we get on success can be visualized as follows:
            // [align_of::<OriginalPointer>, size_of::<OriginalPointer>, layout.align, layout.size]
            // Alignment calculation for the layout starts here ---------^
            // The resulting pointer will point somewhere "within" the `layout.align` region.
            // From this pointer, we subtract `alloc_information_size` which guarantees enough space to store
            // one `OriginalPointer` at its required alignment.

            // Allocate memory.
            let allocated_memory_region: *mut u8 = pvPortMalloc(required_size).cast();

            // We return a null pointer if the allocation failed.
            if allocated_memory_region.is_null() {
                return ptr::null_mut();
            }

            // Calculate the start of `layout.align`:
            // [align_of::<OriginalPointer>, size_of::<OriginalPointer>, !layout.align!, layout.size]
            //
            // SAFETY:
            // We allocated alloc_information_size + layout.align() + layout.size(), which is at least
            // `alloc_information_size` in size.
            let layout_align_start = unsafe { allocated_memory_region.add(alloc_information_size) };

            // Calculate the offset that needs to be applied from the start of `layout.align` to achieve the alignment
            // required by the layout. [align_of::<OriginalPointer>, size_of::<OriginalPointer>,
            // !layout.align!, layout.size]
            let offset = layout_align_start.align_offset(layout.align());

            if offset >= layout.align() {
                // The required alignment cannot be achieved, free memory and return null pointer.
                // We cannot panic here as that would result in undefined behavior.
                //
                // SAFETY:
                // We pass the pointer we received from `pvPortMalloc` straight to `vPortFree`.
                unsafe { vPortFree(allocated_memory_region.cast()) };
                return ptr::null_mut();
            }

            // Calculate the start of the layout memory region (which will be returned from this function).
            //
            // SAFETY:
            // `offset` < `layout.align` which means we have enough space to fit `layout.size`.
            let layout_memory_region = unsafe { layout_align_start.add(offset) };

            // Calculate the start of the memory region intended for the `OriginalPointer`.
            // [!align_of::<OriginalPointer>, size_of::<OriginalPointer>!, layout.align, layout.size]
            //
            // SAFETY:
            // `alloc_information_size` + `offset` >= `alloc_information_size`, which means we are within the allocated
            // memory region.
            let alloc_info_region_start =
                unsafe { layout_memory_region.sub(alloc_information_size) };

            // Calculate the required offset from `alloc_info_region_start` to align `OriginalPointer` correctly.
            let alloc_info_offset =
                alloc_info_region_start.align_offset(align_of::<OriginalPointer>());

            if alloc_info_offset >= align_of::<OriginalPointer>() {
                // The required alignment cannot be achieved, free memory and return null pointer.
                // We cannot panic here as that would result in undefined behavior.
                //
                // SAFETY:
                // We pass the pointer we received from `pvPortMalloc` straight to `vPortFree`.
                unsafe { vPortFree(allocated_memory_region.cast()) };
                return ptr::null_mut();
            }

            // Calculate the address at which we can place `OriginalPointer`.
            //
            // SAFETY:
            // There are at least `alloc_information_size` bytes of space between `alloc_info_region_start` and
            // layout_memory_region. `alloc_info_offset` < align_of::<OriginalPointer>() <
            // `alloc_information_size`
            let original_pointer_location =
                unsafe { alloc_info_region_start.add(alloc_info_offset) };

            // Write `OriginalPointer` to memory.
            //
            // SAFETY:
            // Between `original_pointer_location` and `layout_memory_region` are at least
            // `size_of::<OriginalPointer>()` bytes space: `layout_memory_region` -
            // `alloc_info_region_start` = `alloc_information_size` `alloc_information_size` - `offset` >=
            // `size_of::<OriginalPointer>()` We ensured `original_pointer_location` is aligned correctly
            // for `OriginalPointer`.
            unsafe {
                original_pointer_location
                    .cast::<OriginalPointer>()
                    .write(OriginalPointer(allocated_memory_region));
            }

            layout_memory_region
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= usize::from(portBYTE_ALIGNMENT) {
            // SAFETY:
            // We pass the pointer we received from `pvPortMalloc`.
            unsafe { vPortFree(ptr.cast()) }
        } else {
            let alloc_information_size =
                size_of::<OriginalPointer>() + align_of::<OriginalPointer>();

            // Calculate the start of the memory region where `OriginalPointer` can be placed.
            //
            // SAFETY:
            // In `alloc` we placed `OriginalPointer` starting from the returned pointer minus `alloc_information_size`.
            // Since we are in `dealloc`, the same operation must be valid here.
            let alloc_info_region_start = unsafe { ptr.sub(alloc_information_size) };

            // Since we are in `dealloc`, calculating this offset must have worked in the `alloc` function.
            let alloc_info_offset =
                alloc_info_region_start.align_offset(align_of::<OriginalPointer>());

            // Calculate the location of the `original_pointer_location`.
            //
            // SAFETY:
            // `alloc_info_offset` < `align_of::<OriginalPointer>()` < `alloc_information_size`
            let original_pointer_location =
                unsafe { alloc_info_region_start.add(alloc_info_offset) };

            // Read the original pointer from the memory.
            //
            // SAFETY:
            // In `alloc` we wrote the `OriginalPointer` to this location which makes this valid for reads of
            // `OriginalPointer`.
            let original_pointer =
                unsafe { original_pointer_location.cast::<OriginalPointer>().read().0 };

            // SAFETY:
            // We pass the pointer we received from `pvPortMalloc`.
            unsafe { vPortFree(original_pointer.cast()) }
        }
    }
}
