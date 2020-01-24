use choochoos_sys::Tid;

mod pq {
    generic_containers::impl_priority_queue!(8, 16);
}
use pq::{PriorityQueue, PriorityQueueError};

extern "C" {
    // implemented in asm.s
    fn _activate_task(sp: *mut usize) -> *mut usize;
    fn _swi_handler();

    // provided by userspace
    fn FirstUserTask();
}

// FirstUserTask is technically an `unsafe extern "C" fn` instead of a plain 'ol
// `extern "C" fn`. This trampoline is a zero-cost way to get the types to line
// up correctly.
#[inline]
extern "C" fn first_user_task_trampoline() {
    unsafe { FirstUserTask() }
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

pub struct TaskDescriptor {
    priority: isize,
    // tid: Tid,
    parent_tid: Option<Tid>,
    sp: *mut usize,
}

/// Global kernel singleton
pub static mut KERNEL: Option<Kernel> = None;

pub struct Kernel {
    tasks: [Option<TaskDescriptor>; 8],
    current_tid: Option<Tid>,
    ready_queue: PriorityQueue<Tid>,
}

impl Kernel {
    fn new() -> Kernel {
        Kernel {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: PriorityQueue::new(),
        }
    }

    /// Set up the global kernel context, and prime the kernel to execute it's
    /// first task.
    ///
    /// # Safety
    ///
    /// Must only be called once before the main kernel loop
    pub unsafe fn init() -> &'static mut Kernel {
        KERNEL = Some(Kernel::new());
        let kernel = KERNEL.as_mut().unwrap();

        // register interrupt handlers
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);

        kernel.exec_syscall(Syscall::Create {
            priority: 4,
            function: Some(first_user_task_trampoline),
        });

        kernel
    }

    /// Start the main Kernel loop.
    // TODO: return status code?
    pub fn run(&mut self) {
        loop {
            // determine which tid to schedule next
            let tid = match self.ready_queue.pop() {
                Ok(tid) => {
                    self.current_tid = Some(tid);
                    tid
                }
                Err(PriorityQueueError::Empty) => return,
                Err(e) => panic!("Unexpected error: {:?}", e),
            };

            // activate the task
            let sp = self.tasks[tid.raw()].as_mut().unwrap().sp;
            let next_sp = unsafe { _activate_task(sp) };

            match self.tasks[tid.raw()].as_mut() {
                None => {
                    // task was exited / destroyed
                }
                Some(task) => {
                    task.sp = next_sp;
                    self.ready_queue
                        .push(tid, task.priority as usize)
                        .expect("out of space on the ready queue");
                }
            }
        }
    }

    fn exec_syscall(&mut self, syscall: Syscall) -> isize {
        kdebug!("Called {:x?}", syscall);

        use Syscall::*;
        match syscall {
            Yield => 0,
            Exit => {
                self.tasks[self.current_tid.unwrap().raw()] = None;
                self.current_tid = None;
                0
            }
            MyTid => self
                .current_tid
                .map(|tid| tid.raw() as isize)
                .expect("MyTid syscall cannot return None"),
            MyParentTid => {
                let current_tid = match self.current_tid {
                    Some(tid) => tid,
                    None => return -1,
                };
                self.tasks[current_tid.raw()]
                    .as_ref()
                    .map(|t| t.parent_tid)
                    .flatten()
                    .map(|tid| tid.raw() as isize)
                    .unwrap_or(-1) // implementation dependent
            }
            Create { priority, function } => self.handle_create(priority, function),
        }
    }

    fn get_free_tid(&mut self) -> Option<Tid> {
        // find first available none slot, and return a Tid corresponding to it's index
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, t)| t.is_none())
            .map(|(i, _)| unsafe { Tid::from_raw(i) })
    }

    fn handle_create(&mut self, priority: isize, function: Option<extern "C" fn()>) -> isize {
        let function = match function {
            Some(f) => f,
            // TODO? make this an error code?
            None => panic!("Cannot create task with null pointer"),
        };

        let tid = match self.get_free_tid() {
            Some(tid) => tid,
            None => return -2, // out of tids
        };

        // set up a fresh stack for the new task. This requires some unsafe,
        // low-level shenanigans.
        let sp = unsafe {
            // provided by the linker
            extern "C" {
                static __USER_STACKS_START__: core::ffi::c_void;
            }

            // TODO: find a smarter user stack size number
            const USER_STACK_SIZE: usize = 0x40000;

            let start_of_stack =
                (&__USER_STACKS_START__ as *const _ as usize) + (USER_STACK_SIZE * (tid.raw() + 1));
            let sp = (start_of_stack - core::mem::size_of::<FreshStack>()) as *mut usize;

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

            let stackview = &mut *(sp as *mut FreshStack);
            stackview.dummy_syscall_response = 0xdead_beef;
            stackview.spsr = 0xd0;
            stackview.start_addr = Some(function);
            for (i, r) in &mut stackview.regs.iter_mut().enumerate() {
                *r = i;
            }
            stackview.lr = choochoos_sys::exit;

            sp
        };

        // create the new task descriptor
        self.tasks[tid.raw()] = Some(TaskDescriptor {
            priority,
            // tid,
            parent_tid: self.current_tid,
            sp,
        });

        self.ready_queue
            .push(tid, priority as usize)
            .expect("out of space on the ready queue");

        tid.raw() as isize
    }

    // utility method to retrieve the current_tid. should only be used for debugging
    #[doc(hidden)]
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub(crate) fn current_tid(&self) -> Option<Tid> {
        self.current_tid
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
        .expect("swi handler called before kernel has been initialized")
        .exec_syscall(syscall)
}
