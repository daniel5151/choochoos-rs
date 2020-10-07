use owo_colors::OwoColorize;

#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // prints "userspace panicked at ..."
    ts7200::blocking_println!("{}{}", "userspace ".red(), info.red());
    // TODO: call `Shutdown` syscall
    loop {}
}
