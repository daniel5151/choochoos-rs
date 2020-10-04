use core::ptr;

use heapless::binary_heap::{BinaryHeap, Max};
use heapless::consts::*;

use choochoos_abi as abi;

use abi::{syscall::SyscallNo, Tid};

extern "C" {
    // implemented in asm.s
    fn _activate_task(sp: ptr::NonNull<UserStack>) -> ptr::NonNull<UserStack>;
    fn _swi_handler();
}

/// Provides a structured view into a suspended user stack.
#[repr(C, align(4))]
#[derive(Debug)]
struct UserStack {
    spsr: usize,
    pc: extern "C" fn(),
    regs: [usize; 13],
    lr: usize,
    // gets indexed using `get_unchecked`
    other_params: [usize; 0],
}

impl UserStack {
    // TODO?: support returning structs?
    fn inject_return_value(&mut self, val: usize) {
        self.regs[0] = val;
    }

    fn args(&mut self) -> UserStackArgs<'_> {
        UserStackArgs {
            stack: self,
            idx: 0,
        }
    }
}

/// Helper to extract arguments from a user stack.
pub struct UserStackArgs<'a> {
    stack: &'a UserStack,
    idx: usize,
}

impl<'a> UserStackArgs<'a> {
    // TODO: this could be made safe with some additional logic that checks
    // `size_of::<T>()` and uses the appropriate ARM calling convention.
    unsafe fn extract_ref<T: Copy>(&mut self) -> &'a T {
        let ret = match self.idx {
            0..=3 => &self.stack.regs[self.idx],
            _ => self.stack.other_params.get_unchecked(self.idx - 4),
        };
        self.idx += 1;
        &*(ret as *const usize as *const T)
    }

    unsafe fn extract<T: 'a + Copy>(&mut self) -> T {
        *self.extract_ref()
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

/// There can be only one kernel.
///
/// Ideally, we could have a const `Kernel::new()` method, but unfortunately,
/// Rust's `const` support just ain't there yet. Instead, we use an Option to
/// represent the initial "uninitialized" state, and later use unchecked unwraps
/// to access the underlying value.
///
/// This could potentially be replaced with a UnsafeCell<MaybeUninit<Kernel>>,
/// but this approach is fine for now.
static mut KERNEL: Option<Kernel> = None;

impl Kernel {
    /// Set up the global kernel context, and prime the kernel to execute it's
    /// first task.
    ///
    /// # Safety
    ///
    /// Must only be called once before the main kernel loop
    pub unsafe fn init() -> &'static mut Kernel {
        KERNEL = Some(Kernel {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: BinaryHeap::new(),
        });
        let kernel = KERNEL.as_mut().unwrap();

        // -------- register interrupt handlers -------- //

        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);

        // -------- peripheral hardware init -------- //

        use ts7200::hw::uart;
        let mut term_uart = uart::Uart::new(uart::Channel::COM2);
        term_uart.set_fifo(false);

        // -------- spawn the FirstUserTask -------- //

        // provided by userspace
        extern "C" {
            fn FirstUserTask();
        }

        // FirstUserTask technically has the type `unsafe extern "C" fn` instead of a
        // plain 'ol `extern "C" fn`. This "trampoline" is a zero-cost way to get the
        // types to line up correctly.
        #[inline]
        extern "C" fn first_user_task_trampoline() {
            unsafe { FirstUserTask() }
        }

        kernel.create_task(0, Some(first_user_task_trampoline));

        kernel
    }

    /// Start the main Kernel loop.
    // TODO: return status code?
    pub fn run(&mut self) -> isize {
        loop {
            // determine which tid to schedule next
            let tid = match self.ready_queue.pop() {
                Some(item) => item.tid,
                // TODO: wait for IRQ
                None => return 0,
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
        let stack = sp.as_mut();

        let syscall_no = SyscallNo::from_u8(no).expect("invalid syscall");
        kdebug!("Called {:x?}", syscall_no);

        // package raw stack args into structured enum
        match syscall_no {
            SyscallNo::Yield => self.syscall_yield(stack),
            SyscallNo::Exit => self.syscall_exit(stack),
            SyscallNo::MyParentTid => self.syscall_my_parent_tid(stack),
            SyscallNo::MyTid => self.syscall_my_tid(stack),
            SyscallNo::Create => self.syscall_create(stack),
            other => panic!("unimplemented syscall: {:?}", other),
        };
    }

    unsafe fn handle_interrupt(&mut self) {
        // stubbed
    }

    fn get_free_tid(&mut self) -> Option<Tid> {
        // find first available none slot, and return a Tid corresponding to it's index
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, t)| t.is_none())
            .map(|(i, _)| unsafe { Tid::from_raw(i) })
    }

    fn syscall_yield(&mut self, _stack: &mut UserStack) {}

    fn syscall_exit(&mut self, _stack: &mut UserStack) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        self.tasks[current_tid.raw()] = None;
        self.current_tid = None;
    }

    fn syscall_my_tid(&mut self, stack: &mut UserStack) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        stack.inject_return_value(current_tid.raw())
    }

    fn syscall_my_parent_tid(&mut self, stack: &mut UserStack) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        let ret = self.tasks[current_tid.raw()]
            .as_ref()
            .map(|t| t.parent_tid)
            .flatten()
            .map(|tid| tid.raw() as isize)
            .unwrap_or(-1); // implementation dependent

        stack.inject_return_value(ret as usize)
    }

    fn create_task(&mut self, priority: usize, function: Option<extern "C" fn()>) -> Option<Tid> {
        let function = match function {
            Some(f) => f,
            // TODO? make this an error code?
            None => panic!("Cannot create task with null pointer"),
        };

        let tid = self.get_free_tid()?;

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
            let sp = (start_of_stack - core::mem::size_of::<UserStack>()) as *mut UserStack;

            let mut stackview = &mut *sp;
            stackview.spsr = 0x50;
            stackview.pc = function;
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

        Some(tid)
    }

    fn syscall_create(&mut self, stack: &mut UserStack) {
        let mut args = stack.args();
        let priority = unsafe { args.extract::<usize>() };
        let function = unsafe { args.extract::<Option<extern "C" fn()>>() };

        let ret = match self.create_task(priority, function) {
            Some(tid) => tid.raw() as isize,
            None => -2,
        };

        stack.inject_return_value(ret as _);
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
    match &mut KERNEL {
        Some(kernel) => kernel.handle_syscall(no, sp),
        None => core::hint::unreachable_unchecked(),
    }
}

/// Called by the _irq_handler assembly routine
#[no_mangle]
unsafe extern "C" fn handle_interrupt() {
    match &mut KERNEL {
        Some(kernel) => kernel.handle_interrupt(),
        None => core::hint::unreachable_unchecked(),
    }
}
