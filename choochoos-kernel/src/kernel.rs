use core::ptr;

use heapless::binary_heap::{BinaryHeap, Max};
use heapless::consts::*;

use choochoos_abi as abi;

use abi::{syscall::SyscallNo, Tid};

use crate::user_slice::{self, UserSlice, UserSliceMut};

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

/// Helper to extract arguments from a user's stack.
pub struct UserStackArgs<'a> {
    stack: &'a UserStack,
    idx: usize,
}

impl<'a> UserStackArgs<'a> {
    /// Obtain a reference to the next argument in the user's stack.
    ///
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    unsafe fn extract_ref<T: Copy>(&mut self) -> &'a T {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<usize>());
        let ret = match self.idx {
            0..=3 => &self.stack.regs[self.idx],
            _ => self.stack.other_params.get_unchecked(self.idx - 4),
        };
        self.idx += 1;
        &*(ret as *const usize as *const T)
    }

    /// Obtain a copy of the next argument in the user's stack.
    ///
    /// # Panics
    ///
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    unsafe fn extract<T: 'a + Copy>(&mut self) -> T {
        *self.extract_ref()
    }
}

#[derive(Debug)]
enum TaskState {
    Ready,
    SendWait {
        msg_src: UserSlice<u8>,
        reply_dst: UserSliceMut<u8>,
        next: Option<Tid>,
    },
    RecvWait {
        sender_tid_dst: Option<ptr::NonNull<Tid>>,
        recv_dst: UserSliceMut<u8>,
    },
    ReplyWait {
        reply_dst: UserSliceMut<u8>,
    },
    // EventWait,
}

struct TaskDescriptor {
    priority: isize,
    parent_tid: Option<Tid>,
    sp: ptr::NonNull<UserStack>,

    state: TaskState,

    send_queue_head: Option<Tid>,
    send_queue_tail: Option<Tid>,
}

impl TaskDescriptor {
    pub fn new(
        priority: isize,
        parent_tid: Option<Tid>,
        sp: ptr::NonNull<UserStack>,
    ) -> TaskDescriptor {
        TaskDescriptor {
            priority,
            parent_tid,
            sp,
            state: TaskState::Ready,
            send_queue_head: None,
            send_queue_tail: None,
        }
    }

    /// Forcefully inject a return value into the task's stack.
    ///
    /// # Panics
    ///
    /// Panics if the task is in the `Ready` state.
    pub fn inject_return_value(&mut self, val: usize) {
        if matches!(self.state, TaskState::Ready) {
            panic!("tried to inject return value while task was TaskState::Ready")
        }

        unsafe { self.sp.as_mut() }.inject_return_value(val)
    }
}

/// Implements `Ord` by priority
#[derive(Debug, Eq, PartialEq)]
struct ReadyQueueItem {
    pub priority: isize,
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
    /// Set up the global kernel context.
    pub unsafe fn init() -> &'static mut Kernel {
        if let Some(ref mut kernel) = &mut KERNEL {
            return kernel;
        }

        // Initialize the kernel's static state.
        let kernel = Kernel {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: BinaryHeap::new(),
        };

        // Set the global kernel context.
        KERNEL = Some(kernel);
        let kernel = KERNEL.as_mut().unwrap();

        // Register interrupt handlers
        core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);

        // Initialize key peripheral hardware
        // TODO: this should eventually be done from userspace.
        use ts7200::hw::uart;
        let mut term_uart = uart::Uart::new(uart::Channel::COM2);
        term_uart.set_fifo(false);

        // as a convenience, return a reference to the newly initialized global kernel
        kernel
    }

    /// Start the main Kernel loop.
    // TODO: return status code?
    pub fn run(&mut self) -> isize {
        // -------- spawn the `FirstUserTask` -------- //

        // provided by userspace
        #[link(name = "userspace", kind = "static")]
        extern "C" {
            fn FirstUserTask();
            fn NameServerTask();
        }

        // Tasks have the type `unsafe extern "C" fn` instead of a plain 'ol
        // `extern "C" fn`. These inlined trampolines are a zero-cost way to get
        // the types to line up correctly.
        #[inline(always)]
        extern "C" fn first_user_task_trampoline() {
            unsafe { FirstUserTask() }
        }
        #[inline(always)]
        extern "C" fn nameserver_task_trampoline() {
            unsafe { NameServerTask() }
        }

        // TODO: ensure that `NameServerTask` corresponds to abi::NAMESERVER_TID
        (self.create_task(0, Some(first_user_task_trampoline))).unwrap();
        (self.create_task(0, Some(nameserver_task_trampoline))).unwrap();

        // -------- enter the main kernel loop -------- //

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
            self.current_tid = None;

            // there's a chance that the task was exited / destroyed
            if let Some(ref mut task) = self.tasks[tid.raw()] {
                task.sp = next_sp;

                if matches!(task.state, TaskState::Ready) {
                    self.ready_queue
                        .push(ReadyQueueItem {
                            priority: task.priority,
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

        match syscall_no {
            SyscallNo::Yield => self.syscall_yield(stack),
            SyscallNo::Exit => self.syscall_exit(stack),
            SyscallNo::MyParentTid => self.syscall_my_parent_tid(stack),
            SyscallNo::MyTid => self.syscall_my_tid(stack),
            SyscallNo::Create => self.syscall_create(stack),
            SyscallNo::Send => self.syscall_send(stack),
            SyscallNo::Receive => self.syscall_recieve(stack),
            SyscallNo::Reply => self.syscall_reply(stack),
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
        let task = self.tasks[current_tid.raw()].as_mut().unwrap();

        // unblock any tasks that might be waiting for a response
        if let Some(mut tid) = task.send_queue_head {
            loop {
                let task = self.tasks[tid.raw()].as_mut().unwrap();
                let next_tid = match task.state {
                    TaskState::SendWait { next, .. } => next,
                    _ => panic!(),
                };

                // SRR could not be completed, return -2 to the sender
                task.inject_return_value(-2 as _);
                task.state = TaskState::Ready;
                self.ready_queue
                    .push(ReadyQueueItem {
                        tid,
                        priority: task.priority,
                    })
                    .expect("out of space on the ready queue");

                match next_tid {
                    None => break,
                    Some(next_tid) => tid = next_tid,
                }
            }
        }

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

    fn create_task(
        &mut self,
        priority: isize,
        function: Option<extern "C" fn()>,
    ) -> Result<Tid, isize> {
        let function = match function {
            Some(f) => f,
            // TODO? make this an error code?
            None => panic!("Cannot create task with null pointer"),
        };

        // this is an artificial limitation, but it could come in handy in the future.
        if priority < 0 {
            return Err(-1);
        }

        let tid = self.get_free_tid().ok_or(-2_isize)?;

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

            // HACK: used to run old c-based choochoos programs that assumed a
            // statically linked userspace.
            #[cfg(feature = "legacy-implicit-exit")]
            {
                unsafe extern "C" fn _implicit_exit() {
                    llvm_asm!("swi #1")
                }
                stackview.lr = _implicit_exit as usize;
            }

            ptr::NonNull::new_unchecked(sp)
        };

        // create the new task descriptor
        self.tasks[tid.raw()] = Some(TaskDescriptor::new(priority, self.current_tid, sp));

        self.ready_queue
            .push(ReadyQueueItem { tid, priority })
            .expect("out of space on the ready queue");

        Ok(tid)
    }

    fn syscall_create(&mut self, stack: &mut UserStack) {
        let mut args = stack.args();
        let priority = unsafe { args.extract::<isize>() };
        let function = unsafe { args.extract::<Option<extern "C" fn()>>() };

        let ret = match self.create_task(priority, function) {
            Ok(tid) => tid.raw() as isize,
            Err(code) => code,
        };

        stack.inject_return_value(ret as _);
    }

    fn syscall_reply(&mut self, stack: &mut UserStack) {
        let mut args = stack.args();
        let tid = unsafe { args.extract::<Tid>() };
        let reply_ptr = unsafe { args.extract::<*mut u8>() };
        let reply_len = unsafe { args.extract::<usize>() };

        let reply = if reply_ptr.is_null() {
            UserSlice::empty()
        } else {
            unsafe { user_slice::from_raw_parts(ptr::NonNull::new_unchecked(reply_ptr), reply_len) }
        };

        let ret = match self.reply(tid, reply) {
            Ok(response_len) => response_len,
            Err(code) => code as usize,
        };

        stack.inject_return_value(ret as _)
    }

    fn syscall_recieve(&mut self, stack: &mut UserStack) {
        let mut args = stack.args();
        let sender_tid_dst = unsafe { args.extract::<*mut Tid>() };
        let msg_ptr = unsafe { args.extract::<*mut u8>() };
        let msg_len = unsafe { args.extract::<usize>() };

        let sender_tid_dst = if sender_tid_dst.is_null() {
            None
        } else {
            unsafe { Some(ptr::NonNull::new_unchecked(sender_tid_dst)) }
        };

        let msg = if msg_ptr.is_null() {
            UserSliceMut::empty()
        } else {
            unsafe { user_slice::from_raw_parts_mut(ptr::NonNull::new_unchecked(msg_ptr), msg_len) }
        };

        let ret = match self.recieve(sender_tid_dst, msg) {
            Ok(response_len) => response_len,
            Err(code) => code as usize,
        };

        stack.inject_return_value(ret as _)
    }

    fn syscall_send(&mut self, stack: &mut UserStack) {
        let mut args = stack.args();
        let receiver_tid = unsafe { args.extract::<Tid>() };
        let msg_ptr = unsafe { args.extract::<*mut u8>() };
        let msg_len = unsafe { args.extract::<usize>() };
        let reply_ptr = unsafe { args.extract::<*mut u8>() };
        let reply_len = unsafe { args.extract::<usize>() };

        let msg = if msg_ptr.is_null() {
            UserSlice::empty()
        } else {
            unsafe { user_slice::from_raw_parts(ptr::NonNull::new_unchecked(msg_ptr), msg_len) }
        };

        let reply = if reply_ptr.is_null() {
            UserSliceMut::empty()
        } else {
            unsafe {
                user_slice::from_raw_parts_mut(ptr::NonNull::new_unchecked(reply_ptr), reply_len)
            }
        };

        match self.send(receiver_tid, msg, reply) {
            Ok(_) => {} // value in injected as part of the `Reply` syscall
            Err(code) => stack.inject_return_value(code as _),
        };
    }

    fn send(
        &mut self,
        receiver_tid: Tid,
        msg: UserSlice<u8>,
        reply: UserSliceMut<u8>,
    ) -> Result<(), isize> {
        // ensure that the receiver exists
        if !self
            .tasks
            .get(receiver_tid.raw())
            .map(Option::is_some)
            .unwrap_or(false)
        {
            return Err(-1);
        }

        let sender_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        // Due to Rust's mutable aliasing rules, we can't simply take mut references to
        // both the sender and receiver task at the same time, and are forced to
        // "juggle" between them. After all, there's no reason why a sender couldn't be
        // its' own receiver...
        //
        // To cut down on boilerplate, these macros encapsulate the various
        // array-indexing + unwrapping operations required to get a mutable reference to
        // the sender/receiver tasks.

        macro_rules! receiver {
            () => {
                self.tasks[receiver_tid.raw()].as_mut().unwrap()
            };
        }

        macro_rules! sender {
            () => {
                self.tasks[sender_tid.raw()].as_mut().unwrap();
            };
        }

        let receiver = receiver!();
        match receiver.state {
            TaskState::RecvWait {
                sender_tid_dst,
                mut recv_dst,
            } => {
                let msg_len = msg.len().min(recv_dst.len());
                recv_dst[..msg_len].copy_from_slice(&msg[..msg_len]);

                if let Some(mut sender_tid_dst) = sender_tid_dst {
                    unsafe { *sender_tid_dst.as_mut() = sender_tid };
                }

                receiver.inject_return_value(msg_len);
                receiver.state = TaskState::Ready;
                self.ready_queue
                    .push(ReadyQueueItem {
                        tid: receiver_tid,
                        priority: receiver.priority,
                    })
                    .expect("out of space on the ready queue");

                sender!().state = TaskState::ReplyWait { reply_dst: reply }
            }
            _ => {
                match receiver.send_queue_head {
                    None => {
                        assert!(receiver.send_queue_tail.is_none());
                        receiver.send_queue_head = Some(sender_tid);
                        receiver.send_queue_tail = Some(sender_tid);
                    }
                    Some(_) => {
                        assert!(receiver.send_queue_tail.is_some());

                        let old_tail = self.tasks[receiver.send_queue_tail.unwrap().raw()]
                            .as_mut()
                            .unwrap();
                        match old_tail.state {
                            TaskState::SendWait { ref mut next, .. } => *next = Some(sender_tid),
                            _ => panic!(),
                        };

                        let receiver = receiver!();
                        receiver.send_queue_tail = Some(sender_tid);
                    }
                }

                assert!(matches!(sender!().state, TaskState::Ready));
                sender!().state = TaskState::SendWait {
                    msg_src: msg,
                    reply_dst: reply,
                    next: None,
                };
            }
        }

        Ok(())
    }

    fn recieve(
        &mut self,
        sender_tid_dst: Option<ptr::NonNull<Tid>>,
        mut msg_dst: UserSliceMut<u8>,
    ) -> Result<usize, isize> {
        let receiver_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let receiver = self.tasks[receiver_tid.raw()].as_mut().unwrap();

        if !matches!(receiver.state, TaskState::Ready) {
            panic!(
                "Receive() called from task in non-ready state {:?}",
                receiver.state
            );
        };

        let sender_tid = match receiver.send_queue_head {
            Some(tid) => tid,
            None => {
                receiver.state = TaskState::RecvWait {
                    sender_tid_dst,
                    recv_dst: msg_dst,
                };
                // this will be overwritten when a sender shows up
                return Err(-3);
            }
        };

        let sender = self.tasks[sender_tid.raw()]
            .as_mut()
            .expect("sender was unexpectedly missing");

        let msg_src = match sender.state {
            TaskState::SendWait { msg_src, .. } => msg_src,
            _ => panic!("sender was not in SendWait state"),
        };

        let msg_len = msg_src.len().min(msg_dst.len());

        msg_dst[..msg_len].copy_from_slice(&msg_src[..msg_len]);

        if let Some(mut sender_tid_dst) = sender_tid_dst {
            unsafe {
                *sender_tid_dst.as_mut() = sender_tid;
            }
        }

        let (reply_dst, next) = match sender.state {
            TaskState::SendWait {
                reply_dst, next, ..
            } => (reply_dst, next),
            _ => panic!("sender was not in SendWait state"),
        };
        sender.state = TaskState::ReplyWait { reply_dst };

        let receiver = self.tasks[receiver_tid.raw()].as_mut().unwrap();

        receiver.send_queue_head = next;
        match receiver.send_queue_head {
            None => receiver.send_queue_tail = None,
            Some(_) => assert!(receiver.send_queue_tail.is_some()),
        }

        Ok(msg_len)
    }

    fn reply(&mut self, tid: Tid, reply: UserSlice<u8>) -> Result<usize, isize> {
        let receiver = self
            .tasks
            .get_mut(tid.raw())
            .ok_or(-1_isize)?
            .as_mut()
            .ok_or(-1_isize)?;

        let mut reply_dst = match receiver.state {
            TaskState::ReplyWait { reply_dst } => reply_dst,
            _ => return Err(-2),
        };

        let msg_len = reply_dst.len().min(reply.len());

        reply_dst[..msg_len].copy_from_slice(&reply[..msg_len]);

        // Return the length of the reply to the original sender.
        //
        // The receiver of the reply is blocked, so the stack pointer
        // in the TaskDescriptor points at the top of the stack. Since
        // the top of the stack represents the syscall return word, we
        // can write directly to the stack pointer.
        receiver.inject_return_value(msg_len);
        receiver.state = TaskState::Ready;
        self.ready_queue
            .push(ReadyQueueItem {
                tid,
                priority: receiver.priority,
            })
            .expect("out of space on the ready queue");

        Ok(msg_len)
    }

    /// Utility method to retrieve the current_tid. Only used when the `kdebug`
    /// feature is active.
    #[doc(hidden)]
    #[allow(dead_code)]
    #[cfg(feature = "kdebug")]
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
