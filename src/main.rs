#![no_std]
#![no_main]
#![feature(asm, const_fn, const_if_match)]
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

// These modules need to come first, as they expose useful macros to the rest of
// the crate
#[macro_use]
mod busy_wait_log;
#[macro_use]
mod kernel_log;

mod boilerplate;

pub mod ffi;
pub mod ts7200;
pub mod uart;

use crate::ffi::Void;

#[repr(C)]
#[derive(Debug)]
struct SwiUserStack {
    start_addr: usize,
    spsr: usize,
    regs: [usize; 13],
    lr: usize,
    other_params: [usize; 4],
}

#[no_mangle]
extern "C" fn handle_syscall(no: usize, sp: SwiUserStack) -> isize {
    kdebug!("Hello from the syscall handler!");
    kdebug!("  Called with syscall {}, {:?}", no, sp);

    3
}

pub mod syscalls {
    mod raw {
        extern "C" {
            pub fn Yield();
        }
    }

    pub fn r#yield() {
        unsafe { raw::Yield() }
    }
}

/// Helper POD struct to init new user task stacks
#[repr(C)]
#[derive(Debug)]
struct FreshStack {
    dummy_syscall_response: usize,
    start_addr: fn(),
    spsr: usize,
    regs: [usize; 13],
    lr: usize,
}

// provided by the linker
extern "C" {
    static __USER_STACKS_START__: Void;
}

const USER_STACK_SIZE: usize = 0x40000;

fn create_task(_priority: usize, function: fn()) -> usize {
    // set up memory for the initial user stack
    let start_of_stack = unsafe { (&__USER_STACKS_START__ as *const _ as usize) + USER_STACK_SIZE };
    let top_of_stack = start_of_stack - core::mem::size_of::<FreshStack>();

    let stack = unsafe { &mut *(top_of_stack as *mut usize as *mut FreshStack) };

    stack.dummy_syscall_response = 0xdead_beef;
    stack.spsr = 0xc0;
    stack.start_addr = function;
    for r in &mut stack.regs {
        *r = 0;
    }
    stack.lr = 0;

    top_of_stack
}

pub fn dummy_task() {
    blocking_println!("Hello from user space!");
    syscalls::r#yield();
}

extern "C" {
    fn _activate_task(sp: usize) -> usize;
    fn _swi_handler();
}

fn main() -> isize {
    // Mess around with the UART
    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);

    // set swi handler
    unsafe {
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);
    }

    kprintln!("Hello from the kernel!");

    let sp = create_task(3, dummy_task);
    kprintln!("Activating task with sp {:#x?}", sp);
    let next_sp = unsafe { _activate_task(sp) };
    kprintln!("Returned from task, new sp at {:#x?}", next_sp);

    kprintln!("Goodbye from the kernel!");

    0
}
