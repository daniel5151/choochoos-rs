use core::ffi::c_void;

/// The first function called by the bootloader. On most platforms, this would
/// return `!`, but the `ts7200` emulator supports returning a return value.
#[no_mangle]
unsafe extern "C" fn _start() -> isize {
    // save the return address on the stack, as super::REDBOOT_RETURN_ADDRESS
    // might be placed in .bss, which is cleared later on in this function!
    let mut redboot_return_address: *const c_void;
    asm!(
        "mov {}, lr",
        out(reg) redboot_return_address
    );

    // provided by the linker
    extern "C" {
        static mut __BSS_START__: c_void;
        static mut __BSS_END__: c_void;
        static mut __HEAP_START__: c_void;
        static mut __HEAP_SIZE__: usize;
    }

    // zero bss
    let mut bss_start = &mut __BSS_START__ as *mut _ as *mut u8;
    while bss_start < (&mut __BSS_END__ as *mut _ as *mut u8) {
        // must be volatile, or else this gets optimized out
        core::ptr::write_volatile(bss_start, 0);
        bss_start = bss_start.offset(1);
    }

    #[cfg(feature = "heap")]
    crate::heap::ALLOCATOR.init(&mut __HEAP_START__ as *mut _ as usize, __HEAP_SIZE__);

    super::REDBOOT_RETURN_ADDRESS = redboot_return_address;

    // HACK: UART init really aught to be done in userspace!
    // We do it here since the kernel currently uses busy-wait logging.
    use ts7200::hw::uart;
    let mut term_uart = uart::Uart::new(uart::Channel::COM2);
    term_uart.set_fifo(false);

    crate::main()
}
