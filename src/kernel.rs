use choochoos_sys::{TaskFn, Tid};

use crate::scheduler::Scheduler;
use crate::KERNEL;

extern "C" {
    fn _activate_task(sp: *mut usize) -> *mut usize;
    fn _swi_handler();
}

#[derive(Debug)]
pub enum Syscall {
    Yield,
    Exit,
    MyTid,
    MyParentTid,
    Create {
        priority: isize,
        function: Option<extern "C" fn()>,
    },
}

pub struct Kernel {
    scheduler: Scheduler,
}

impl Kernel {
    fn new() -> Kernel {
        Kernel {
            scheduler: Scheduler::new(),
        }
    }

    /// # Safety
    ///
    /// Must only be called once before the main kernel loop
    pub unsafe fn init(first_task: TaskFn) -> &'static mut Kernel {
        KERNEL = Some(Kernel::new());
        let kernel = KERNEL.as_mut().unwrap();

        // register swi handler
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);

        kernel.exec_syscall(Syscall::Create {
            priority: 4,
            function: Some(first_task),
        });

        kernel
    }

    pub fn schedule(&mut self) -> Option<Tid> {
        self.scheduler.schedule()
    }

    pub fn activate_task(&mut self, tid: Tid) {
        let sp = self.scheduler.get_sp_mut(tid);
        let next_sp = unsafe { _activate_task(sp) };
        self.scheduler.on_yield(tid, next_sp)
    }

    pub fn exec_syscall(&mut self, syscall: Syscall) -> isize {
        kdebug!("Called {:x?}", syscall);

        use Syscall::*;
        match syscall {
            Yield => 0,
            Exit => {
                self.scheduler.exit_current_task();
                0
            }
            MyTid => self
                .scheduler
                .current_tid()
                .map(|tid| tid.raw() as isize)
                .expect("MyTid syscall cannot return None"),
            MyParentTid => self
                .scheduler
                .current_parent_tid()
                .map(|tid| tid.raw() as isize)
                .unwrap_or(-1), // implementation dependent
            Create { priority, function } => self.handle_create(priority, function),
        }
    }

    fn handle_create(&mut self, priority: isize, function: Option<extern "C" fn()>) -> isize {
        let function = match function {
            Some(f) => f,
            // TODO? make this an error code?
            None => panic!("Cannot create task with null pointer"),
        };

        // provided by the linker
        extern "C" {
            static __USER_STACKS_START__: core::ffi::c_void;
        }

        // TODO: find a smarter user stack size number
        const USER_STACK_SIZE: usize = 0x40000;

        /// Helper POD struct to init new user task stacks
        #[repr(C)]
        #[derive(Debug)]
        struct FreshStack {
            dummy_syscall_response: usize,
            start_addr: Option<extern "C" fn()>,
            spsr: usize,
            regs: [usize; 13],
            lr: fn(),
        }

        let tid = unsafe {
            self.scheduler.new_task(priority, |tid| {
                // set the td's sp to point the new stack
                let start_of_stack = (&__USER_STACKS_START__ as *const _ as usize)
                    + (USER_STACK_SIZE * (tid.raw() + 1));
                let sp = (start_of_stack - core::mem::size_of::<FreshStack>()) as *mut usize;

                // set up memory for the initial user stack
                let stackview = &mut *(sp as *mut FreshStack);
                stackview.dummy_syscall_response = 0xdead_beef;
                stackview.spsr = 0xd0;
                stackview.start_addr = Some(function);
                for (i, r) in &mut stackview.regs.iter_mut().enumerate() {
                    *r = i;
                }
                stackview.lr = choochoos_sys::exit;

                sp
            })
        };

        // ask the scheduler to return a fresh tid (and associated stack)
        match tid {
            Some(tid) => tid.raw() as isize,
            None => -2, // out of tids
        }
    }

    /// only for debugging
    #[doc(hidden)]
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub fn current_tid(&self) -> Option<Tid> {
        self.scheduler.current_tid()
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
    other_params: [usize; 4], // 4 could be bumped up, if more args are required
}

/// Called by the _swi_handler assembly routine
#[no_mangle]
unsafe extern "C" fn handle_syscall(no: usize, sp: *const SwiUserStack) -> isize {
    let sp = &*sp;

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

    KERNEL
        .as_mut()
        .expect("swi handler executed before kernel initialization (somehow)")
        .exec_syscall(syscall)
}
