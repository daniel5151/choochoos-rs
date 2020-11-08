use owo_colors::OwoColorize;

// no atomics
static mut RECURSIVE_PANIC: bool = false;

/// Busy-wait prints the error message, and then yields control back to Redboot.
#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // prints "kernel panicked at ..."
    ts7200::bwprintln!("{}{}", "kernel ".red(), info.red());

    // TODO?: flush kernel logs
    // TODO?: manual backtrace / crash dump

    // try and clean up, while making sure that if the teardown routine panics,
    // there isn't an infinite loop.
    unsafe {
        if !RECURSIVE_PANIC {
            RECURSIVE_PANIC = true;
            super::super::teardown();
        }
    }

    // exit to RedBoot
    unsafe {
        asm!(
            "mov pc, {}",
            in(reg) super::REDBOOT_RETURN_ADDRESS,
        );
    }

    // unreachable
    loop {}
}
