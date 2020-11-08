//! Kernel heap implementation. Requires enabling the `heap` feature.

use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::cell::UnsafeCell;

use linked_list_allocator::Heap;

/// The global Kernel allocator.
#[global_allocator]
pub static ALLOCATOR: LLHeap = LLHeap::new();

#[cfg_attr(not(test), alloc_error_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn alloc_error(layout: Layout) -> ! {
    panic!("Kernel heap allocation error: {:?}", layout);
}

/// Wrapper around [`linked_list_allocator::Heap`](https://docs.rs/linked_list_allocator/0.8.6/linked_list_allocator/struct.Heap.html)
/// implementing the [`GlobalAlloc`] trait.
pub struct LLHeap(UnsafeCell<Heap>);

/// # Safety
///
/// The TS-7200 is single core system, so there can't be simultaneous accesses
/// to the `LLHeap`. This marker implementation is required to register `LLHeap`
/// as a global allocator.
unsafe impl Sync for LLHeap {}

impl LLHeap {
    /// Create a new uninitialized `LLHeap`
    pub const fn new() -> LLHeap {
        LLHeap(UnsafeCell::new(Heap::empty()))
    }

    /// Initialize the `LLHeap`, specifying it's start address and size
    ///
    /// # Safety
    ///
    /// This function must be called at most once, and must only be used on an
    /// empty heap.
    pub unsafe fn init(&self, start: usize, size: usize) {
        (*self.0.get()).init(start, size)
    }
}

unsafe impl GlobalAlloc for LLHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self.0.get())
            .allocate_first_fit(layout)
            .ok()
            .map_or(core::ptr::null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self.0.get()).deallocate(core::ptr::NonNull::new_unchecked(ptr), layout)
    }
}
