#![no_std]
#![no_main]
#![feature(asm)] // TODO: get rid of this at some point
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

#[macro_use]
extern crate ts7200;

pub use choochoos_sys::{TaskFn, Tid};

// These modules need to come first, as they expose useful macros to the rest of
// the crate

#[macro_use]
mod kernel_log;

mod boilerplate;
mod kernel;
mod scheduler;
mod syscalls;

use kernel::Kernel;
use scheduler::Scheduler;

pub extern "C" fn dummy_task() {
    blocking_println!("Hello from user space!");
    choochoos_sys::r#yield();
    blocking_println!("Hello once again from user space!");
    // implicit return
}

fn hardware_init() {
    use ts7200::hw::uart;

    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);
}

fn main() -> isize {
    hardware_init();

    kprintln!("Hello from the kernel!");

    // init the kernel with the first user task
    let kern = unsafe { Kernel::init(dummy_task) };

    // let 'er rip
    while let Some(tid) = kern.schedule() {
        kern.activate_task(tid);
    }

    kprintln!("Goodbye from the kernel!");

    0
}
