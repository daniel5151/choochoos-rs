use core::ptr;

use heapless::binary_heap::{BinaryHeap, Max};
use heapless::consts::*;

use choochoos_abi as abi;

use abi::{syscall::SyscallNo, Tid};

extern "C" {
    // implemented in asm.s
    fn _activate_task(sp: ptr::NonNull<UserStack>) -> ptr::NonNull<UserStack>;
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

/// Helper struct to provide a structured representation of the user stack
#[repr(C)]
#[derive(Debug)]
struct UserStack {
    spsr: usize,
    start_addr: extern "C" fn(),
    regs: [usize; 13],
    lr: usize,
    other_params: [usize; 4], // 4 could be bumped up, if more args are required
}

impl UserStack {
    fn fresh_stack_size() -> usize {
        core::mem::size_of::<UserStack>() - core::mem::size_of::<[usize; 4]>()
    }

    fn inject_return_value(&mut self, r0: usize) {
        self.regs[0] = r0;
    }
}

pub struct TaskDescriptor {
    priority: usize,
    // tid: Tid,
    parent_tid: Option<Tid>,
    sp: ptr::NonNull<UserStack>,
}

/// Implements `Ord` by priority
#[derive(Debug, Eq, PartialEq)]
struct ReadyQueueItem {
    pub priority: usize,
    pub tid: Tid,
}

impl PartialOrd for ReadyQueueItem {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for ReadyQueueItem {
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&rhs.priority)
    }
}

/// Global kernel singleton
pub static mut KERNEL: Option<Kernel> = None;

// oh const generics, please land soon
#[allow(non_camel_case_types)]
type MAX_TASKS = U16;
const MAX_TASKS: usize = 16;

/// The core choochoos kernel!
pub struct Kernel {
    tasks: [Option<TaskDescriptor>; MAX_TASKS],
    current_tid: Option<Tid>,
    ready_queue: BinaryHeap<ReadyQueueItem, MAX_TASKS, Max>, // matches number of tasks
}

impl Kernel {
    fn new() -> Kernel {
        Kernel {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: BinaryHeap::new(),
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

        kernel.syscall_create(0, Some(first_user_task_trampoline));

        kernel
    }

    /// Start the main Kernel loop.
    // TODO: return status code?
    pub fn run(&mut self) {
        loop {
            // determine which tid to schedule next
            let tid = match self.ready_queue.pop() {
                Some(item) => item.tid,
                // TODO: wait for IRQ
                None => return,
            };

            // activate the task
            self.current_tid = Some(tid);
            let sp = self.tasks[tid.raw()].as_mut().unwrap().sp;
            let next_sp = unsafe { _activate_task(sp) };

            match self.tasks[tid.raw()].as_mut() {
                None => {
                    // task was exited / destroyed
                }
                Some(task) => {
                    task.sp = next_sp;
                    self.ready_queue
                        .push(ReadyQueueItem {
                            priority: task.priority as usize,
                            tid,
                        })
                        .expect("out of space on the ready queue");
                }
            }
        }
    }

    unsafe fn handle_syscall(&mut self, no: u8, sp: *mut UserStack) {
        let mut sp = ptr::NonNull::new(sp).expect("passed null sp to handle_syscall");
        let sp = sp.as_mut();

        let syscall_no = SyscallNo::from_u8(no).expect("invalid syscall");
        kdebug!("Called {:x?}", syscall_no);

        // package raw stack args into structured enum
        let ret = match syscall_no {
            SyscallNo::Yield => {
                self.syscall_yield();
                None
            }
            SyscallNo::Exit => {
                self.syscall_exit();
                None
            }
            SyscallNo::MyParentTid => {
                let tid = self.syscall_my_parent_tid();
                Some(tid)
            }
            SyscallNo::MyTid => {
                let tid = self.syscall_my_tid();
                Some(tid)
            }
            SyscallNo::Create => {
                let tid = self.syscall_create(
                    core::mem::transmute(sp.regs[0]),
                    core::mem::transmute(sp.regs[1]),
                );
                Some(tid)
            }
            other => panic!("unimplemented syscall: {:?}", other),
        };

        if let Some(ret) = ret {
            sp.inject_return_value(ret as usize);
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

    fn syscall_yield(&mut self) {}

    fn syscall_exit(&mut self) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        self.tasks[current_tid.raw()] = None;
        self.current_tid = None;
    }

    fn syscall_my_tid(&mut self) -> isize {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        current_tid.raw() as _
    }

    fn syscall_my_parent_tid(&mut self) -> isize {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        let tid = self.tasks[current_tid.raw()]
            .as_ref()
            .map(|t| t.parent_tid)
            .flatten()
            .map(|tid| tid.raw() as isize)
            .unwrap_or(-1); // implementation dependent
        tid
    }

    fn syscall_create(&mut self, priority: usize, function: Option<extern "C" fn()>) -> isize {
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
            let sp = (start_of_stack - UserStack::fresh_stack_size()) as *mut UserStack;

            let mut stackview = &mut *sp;
            stackview.spsr = 0x50;
            stackview.start_addr = function;
            for (i, r) in &mut stackview.regs.iter_mut().enumerate() {
                *r = i;
            }
            stackview.lr = 0xffffffff; // will trigger an error in `ts7200` emulator

            ptr::NonNull::new_unchecked(sp)
        };

        // create the new task descriptor
        self.tasks[tid.raw()] = Some(TaskDescriptor {
            priority,
            // tid,
            parent_tid: self.current_tid,
            sp,
        });

        self.ready_queue
            .push(ReadyQueueItem { tid, priority })
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

/// Called by the _swi_handler assembly routine
#[no_mangle]
unsafe extern "C" fn handle_syscall(no: u8, sp: *mut UserStack) {
    KERNEL
        .as_mut()
        .expect("swi handler called before kernel has been initialized")
        .handle_syscall(no, sp)
}

#[no_mangle]
unsafe extern "C" fn handle_interrupt() {
    // stubbed
}