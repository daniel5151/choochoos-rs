//! Types and structures exposed by the `choochoos` kernel.

#![no_std]
#![deny(missing_docs)]

/// Function signature which can be spawned by the kernel.
pub type TaskFn = extern "C" fn();

/// Task descriptor handle.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Tid(usize);

impl Tid {
    /// Create a new Tid from a raw value.
    ///
    /// # Safety
    ///
    /// `val` must correspond to a valid task descriptor.
    pub unsafe fn from_raw(val: usize) -> Tid {
        Tid(val)
    }

    /// Return the Tid's raw value.
    pub fn raw(self) -> usize {
        self.0
    }
}

/// TID of automatically spawned nameserver.
pub const NAMESERVER_TID: Tid = Tid(1);

/// Kernel syscall interface.
pub mod syscall {
    /// Raw `choochoos` syscall numbers (corresponding to calling `swi #x`).
    #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
    #[repr(u8)]
    #[allow(missing_docs)]
    pub enum SyscallNo {
        Yield       = 0,
        Exit        = 1,
        MyParentTid = 2,
        MyTid       = 3,
        Create      = 4,
        Send        = 5,
        Receive     = 6,
        Reply       = 7,
        AwaitEvent  = 8,
        Perf        = 9,
        Shutdown    = 10,
    }

    impl SyscallNo {
        /// Return enum corresponding to raw syscall number (if one exists).
        pub fn from_u8(no: u8) -> Option<SyscallNo> {
            if no > 10 {
                None
            } else {
                // SAFETY: SyscallNo is repr(u8), and was checked to be in bounds
                Some(unsafe { core::mem::transmute(no) })
            }
        }
    }

    /// Function signatures associated with each syscall.
    pub mod signature {
        #![allow(missing_docs)]

        use crate::Tid;

        pub type Yield = unsafe extern "C" fn();
        pub type Exit = unsafe extern "C" fn();
        pub type MyParentTid = unsafe extern "C" fn() -> isize;
        pub type MyTid = unsafe extern "C" fn() -> isize;
        pub type Create =
            unsafe extern "C" fn(priority: isize, function: Option<extern "C" fn()>) -> isize;
        pub type Send = unsafe extern "C" fn(
            tid: Tid,
            msg: *const u8,
            msglen: usize,
            reply: *mut u8,
            rplen: usize,
        ) -> isize;
        pub type Receive =
            unsafe extern "C" fn(tid: *mut Tid, msg: *mut u8, msglen: usize) -> isize;
        pub type Reply = unsafe extern "C" fn(tid: Tid, reply: *const u8, rplen: usize) -> isize;
    }

    /// Errors associated with various syscalls.
    pub mod error {
        /// Errors returned by the `MyParentTid` syscall
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum MyParentTid {
            /// Task does not have a parent.
            NoParent = -1,
        }

        /// Errors returned by the `Create` syscall
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum Create {
            /// Tried to create a task with an invalid priority.
            InvalidPriority      = -1,
            /// The Kernel has run out of task descriptors.
            OutOfTaskDescriptors = -2,
        }

        /// Errors returned by the `Send` syscall.
        ///
        /// Note that a Truncation error will _not_ return an error code, and
        /// must be inferred by checking the length of message returned by the
        /// syscall compared to the expected length.
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum Send {
            /// `tid` is not the task id of an existing task.
            TidDoesNotExist = -1,
            /// The send-receive-reply transaction could not be completed.
            CouldNotSSR     = -2,
        }

        /// Errors returned by the `Reply` syscall.
        ///
        /// Note that a Truncation error will _not_ return an error code, and
        /// must be inferred by checking the length of message returned by the
        /// syscall compared to the expected length.
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum Reply {
            /// `tid` is not the task id of an existing task.
            TidDoesNotExist      = -1,
            /// `tid` is not the task id of a reply-blocked task.
            TidIsNotReplyBlocked = -2,
        }
    }
}
