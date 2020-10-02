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
unsafe extern "C" fn _start() -> isize {
    // save the return address locally, as super::REDBOOT_RETURN_ADDRESS might
    // be placed in .bss, which we clear later in this function!
    let redboot_return_address: *const core::ffi::c_void;
    llvm_asm!("mov $0, lr" : "=r" (redboot_return_address) ::: "volatile");

    // provided by the linker
    extern "C" {
        static mut __BSS_START__: Void;
        static mut __BSS_END__: Void;
        static mut __HEAP_START__: Void;
        static mut __HEAP_SIZE__: Void;
    }

    r0::zero_bss(&mut __BSS_START__, &mut __BSS_END__);

    #[cfg(feature = "heap")]
    super::heap::ALLOCATOR.init(
        &mut __HEAP_START__ as *mut _ as usize,
        &mut __HEAP_SIZE__ as *mut _ as usize,
    );

    super::REDBOOT_RETURN_ADDRESS = redboot_return_address;

    crate::main()
}
