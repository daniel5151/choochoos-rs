#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // move top left + clear line
    blocking_println!("\x1b[H\x1b[2K{}", info);

    // TODO?: flush kernel logs
    // TODO?: manual backtrace / crash dump

    // TODO: exit to RedBoot
    loop {}
}

/// Mirrors the core::ffi::c_void type, but adding a Copy derive
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

    #[cfg(feature = "heap")]
    crate::heap::ALLOCATOR.init(
        &mut __HEAP_START__ as *mut _ as usize,
        &mut __HEAP_SIZE__ as *mut _ as usize,
    );

    crate::main()
}
