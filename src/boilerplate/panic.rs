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
