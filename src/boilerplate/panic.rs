#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // move top left + clear line
    blocking_println!("\x1b[H\x1b[2Kkernel {}", info);

    // TODO?: flush kernel logs
    // TODO?: manual backtrace / crash dump

    // exit to RedBoot
    unsafe {
        asm!("mov r0, #1
              mov pc, $0"
            : // no outputs
            : "r" (super::REDBOOT_RETURN_ADDRESS)
            : "r0"
            : "volatile");
    }

    unreachable!()
}
