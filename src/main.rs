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
mod scheduler;

// FirstUserTask is technically an `unsafe extern "C" fn` instead of a plain 'ol
// `extern "C" fn`. This trampoline is a zero-cost way to get the types to line
// up correctly.
#[inline]
extern "C" fn first_user_task_trampoline() {
    extern "C" {
        fn FirstUserTask();
    }

    unsafe { FirstUserTask() }
}

fn hardware_init() {
    use ts7200::hw::uart;

    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);
}

pub static mut KERNEL: Option<kernel::Kernel> = None;

fn main() -> isize {
    hardware_init();

    kprintln!("Hello from the kernel!");

    // init the kernel with the first user task
    let kern = unsafe { kernel::Kernel::init(first_user_task_trampoline) };

    // let 'er rip
    while let Some(tid) = kern.schedule() {
        kern.activate_task(tid);
    }

    kprintln!("Goodbye from the kernel!");

    0
}
