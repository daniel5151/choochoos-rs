#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let term_uart = unsafe { crate::uart::Uart::new(crate::uart::Channel::COM2) };
    term_uart.write_blocking(b"PANIC!\n\r");

    // TODO: exit to RedBoot?
    loop {}
}

#[no_mangle]
unsafe extern "C" fn __start() -> isize {
    // provided by the linker
    extern "C" {
        static mut __BSS_START__: u32;
        static mut __BSS_END__: u32;
    }

    r0::zero_bss(&mut __BSS_START__, &mut __BSS_END__);
    crate::main()
}
