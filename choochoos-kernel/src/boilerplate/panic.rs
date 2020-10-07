// FIXME: make panic work properly in user code.
//
// Since we directly link with userland, calling `panic!` from user code will
// end up running this method.
//
// One option is to create a different panic! function just for userland, which
// might route to a new syscall (Terminate? Panic?), allowing the kernel to
// ack. the panic, and clean-up accordingly.
//
// Another option would be to avoid using a different macro, and instead add a
// inline-asm check in this method to cause a swi if the method is called from
// userland. Kinda jank, but it should work.

use owo_colors::OwoColorize;

#[cfg_attr(not(test), panic_handler)]
#[allow(dead_code, clippy::empty_loop)]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // prints "kernel panicked at ..."
    ts7200::blocking_println!("{}{}", "kernel ".red(), info.red());

    // TODO?: flush kernel logs
    // TODO?: manual backtrace / crash dump

    // exit to RedBoot
    unsafe {
        llvm_asm!("mov r0, #1
              mov pc, $0"
            : // no outputs
            : "r" (super::REDBOOT_RETURN_ADDRESS)
            : "r0"
            : "volatile");
    }

    loop {}
}
