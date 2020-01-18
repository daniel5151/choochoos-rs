#![no_std]
#![no_main]
#![feature(asm, const_fn, const_if_match)]
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

pub use choochoos_sys::{TaskFn, Tid};

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

mod scheduler;
mod syscalls;

use scheduler::Scheduler;

extern "C" {
    fn _activate_task(sp: *mut usize) -> *mut usize;
    fn _swi_handler();
}

pub struct TaskDescriptor {
    priority: isize,
    tid: Tid,
    parent_tid: Option<Tid>,
    sp: *mut usize,
}

enum Syscall {
    Yield,
    Exit,
    MyTid,
    MyParentTid,
    Create {
        priority: isize,
        function: Option<extern "C" fn()>,
    },
}

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
unsafe extern "C" fn handle_syscall(no: usize, sp: SwiUserStack) -> isize {
    kdebug!("Hello from the syscall handler!");
    kdebug!("  Called with syscall {}, {:?}", no, sp);

    // marshall arguments into the correct structure
    use Syscall::*;
    let syscall = match no {
        0 => Yield,
        1 => Exit,
        2 => MyTid,
        3 => MyParentTid,
        4 => Create {
            priority: core::mem::transmute(sp.regs[0]),
            function: core::mem::transmute(sp.regs[1]),
        },
        _ => panic!("Invalid syscall number"),
    };

    let kernel = &mut *KERNEL;
    kernel.handle_syscall(syscall)
}

pub struct Kernel {
    scheduler: Scheduler,
}

static mut KERNEL: *mut Kernel = core::ptr::null_mut();

impl Kernel {
    pub fn new(first_task: TaskFn) -> Kernel {
        let mut kernel = Kernel {
            scheduler: Scheduler::new(),
        };

        kernel.handle_syscall(Syscall::Create {
            priority: 3,
            function: Some(first_task),
        });

        kernel
    }

    fn schedule(&mut self) -> Option<Tid> {
        self.scheduler.schedule()
    }

    fn activate(&mut self, tid: Tid) {
        let sp = self.scheduler.get_sp_mut(tid);
        let next_sp = unsafe { _activate_task(sp) };
        self.scheduler.on_yield(tid, next_sp)
    }

    // TODO: change return value to result
    fn handle_syscall(&mut self, sysall: Syscall) -> isize {
        use Syscall::*;
        match sysall {
            Yield => syscalls::r#yield(),
            Exit => syscalls::exit(&mut self.scheduler),
            MyTid => unimplemented!(),
            MyParentTid => unimplemented!(),
            Create { priority, function } => {
                return syscalls::create(&mut self.scheduler, priority, function)
            }
        }

        0
    }
}

pub extern "C" fn dummy_task() {
    blocking_println!("Hello from user space!");
    choochoos_sys::r#yield();
    blocking_println!("Hello once again from user space!");
    // implicit return
}

fn hardware_init() {
    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);
}

fn main() -> isize {
    hardware_init();

    kprintln!("Hello from the kernel!");

    // register swi handler
    unsafe {
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);
    }

    // init the kernel with the first user task
    let mut kern = Kernel::new(dummy_task);
    unsafe {
        KERNEL = &mut kern;
    }

    // let 'er rip
    while let Some(tid) = kern.schedule() {
        kern.activate(tid);
    }

    kprintln!("Goodbye from the kernel!");

    0
}
