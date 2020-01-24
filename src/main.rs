#![no_std]
#![no_main]
#![feature(asm)] // TODO: get rid of this at some point?
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

// ensure the userspace is linked in
extern crate userspace;

#[macro_use]
extern crate ts7200;

#[macro_use]
mod kernel_log;

mod boilerplate;
mod kernel;

fn hardware_init() {
    use ts7200::hw::uart;

    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);
}

fn main() -> isize {
    hardware_init();

    // init the kernel with the first user task
    let kern = unsafe { kernel::Kernel::init() };

    // let the kernel do it's thing!
    kern.run();

    0
}
