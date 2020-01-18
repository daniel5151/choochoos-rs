use choochoos_sys::{TaskFn, Tid};

use crate::scheduler::Scheduler;
use crate::syscalls::{self, Syscall};

static mut KERNEL: Kernel = Kernel::new();

extern "C" {
    fn _activate_task(sp: *mut usize) -> *mut usize;
    fn _swi_handler();
}

pub struct Kernel {
    scheduler: Scheduler,
}

impl Kernel {
    const fn new() -> Kernel {
        Kernel {
            scheduler: Scheduler::new(),
        }
    }

    /// # Safety
    ///
    /// Must only be called once before the main kernel loop
    pub unsafe fn init(first_task: TaskFn) -> &'static mut Kernel {
        // register swi handler
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);

        KERNEL.exec_syscall(Syscall::Create {
            priority: 3,
            function: Some(first_task),
        });

        &mut KERNEL
    }

    pub fn schedule(&mut self) -> Option<Tid> {
        self.scheduler.schedule()
    }

    pub fn activate_task(&mut self, tid: Tid) {
        let sp = self.scheduler.get_sp_mut(tid);
        let next_sp = unsafe { _activate_task(sp) };
        self.scheduler.on_yield(tid, next_sp)
    }

    // TODO: change return value to result
    pub fn exec_syscall(&mut self, sysall: Syscall) -> isize {
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

/// Helper struct to provide a structured representation of the user stack, as
/// provided by the _swi_handler routine
#[repr(C)]
#[derive(Debug)]
struct SwiUserStack {
    start_addr: usize,
    spsr: usize,
    regs: [usize; 13],
    lr: usize,
    other_params: [usize; 4],
}

/// Called by the _swi_handler assembly routine
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

    // Safety: _swi_handler will only be called after the kernel is initialized
    KERNEL.exec_syscall(syscall)
}
