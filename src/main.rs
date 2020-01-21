#![no_std]
#![no_main]
#![feature(asm)] // TODO: get rid of this at some point
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
mod scheduler;
mod syscalls;

use kernel::Kernel;
use scheduler::Scheduler;

// init_task is technically an `unsafe extern "C" fn`, instead of a plain 'ol
// `extern "C" fn`, so this zero-cost trampoline has to be used to make the
// signatures line up.
#[inline]
extern "C" fn init_task_trampoline() {
    extern "C" {
        fn init_task();
    }

    unsafe { init_task() }
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
    let kern = unsafe { Kernel::init(init_task_trampoline) };

    // let 'er rip
    while let Some(tid) = kern.schedule() {
        kern.activate_task(tid);
    }

    kprintln!("Goodbye from the kernel!");

    0
}
