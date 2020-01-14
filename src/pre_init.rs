use alloc::alloc::Layout;
use core::alloc::GlobalAlloc;
use core::cell::UnsafeCell;

use linked_list_allocator::Heap;

#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let term_uart = unsafe { crate::uart::Uart::new(crate::uart::Channel::COM2) };
    term_uart.write_blocking(b"PANIC!\n\r");

    // TODO: exit to RedBoot?
    loop {}
}

#[cfg_attr(not(test), alloc_error_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn alloc_error(_layout: Layout) -> ! {
    let term_uart = unsafe { crate::uart::Uart::new(crate::uart::Channel::COM2) };
    term_uart.write_blocking(b"ALLOCATION ERROR!\n\r");

    // TODO: exit to RedBoot?
    loop {}
}

/// Wrapper around [linked_list_allocator::Heap] implementing the GlobalAlloc
/// trait.
struct LLHeap(UnsafeCell<Heap>);

/// # Safety
///
/// The TS-7200 is single core system, so there can't be simultaneous accesses
/// to the LLHeap. This marker implementation is only provided to LLHeap can be
/// registered as a global allocator.
unsafe impl Sync for LLHeap {}

impl LLHeap {
    /// Create a new uninitialized LLHeap
    pub const fn new() -> LLHeap {
        LLHeap(UnsafeCell::new(Heap::empty()))
    }

    /// Initialize the LLHeap, specifying it's start address and size
    ///
    /// # Safety
    ///
    /// This function must be called at most once and must only be used on an
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

#[global_allocator]
static ALLOCATOR: LLHeap = LLHeap::new();

/// Mirrors the core::ffi::c_void type, adding a Copy derive
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum Void {
    #[doc(hidden)]
    Variant1,
    #[doc(hidden)]
    Variant2,
}

#[no_mangle]
unsafe extern "C" fn __start() -> isize {
    // provided by the linker
    extern "C" {
        static mut __BSS_START__: Void;
        static mut __BSS_END__: Void;
        static mut __HEAP_START__: Void;
        static mut __HEAP_SIZE__: Void;
    }

    r0::zero_bss(&mut __BSS_START__, &mut __BSS_END__);
    ALLOCATOR.init(
        &mut __HEAP_START__ as *mut _ as usize,
        &mut __HEAP_SIZE__ as *mut _ as usize,
    );

    crate::main()
}
