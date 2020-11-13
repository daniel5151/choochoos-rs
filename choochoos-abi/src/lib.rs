//! Types and constants which define the choochoos kernel/userspace ABI.
//!
//! This crate is the "source of truth" for the choochoos kernel-userspace ABI,
//! and is used on both sides of the kernel/userspace boundary to ensure that
//! syscall numbers, signatures, errors, etc... are correctly matched.
//!
//! The `choochoos` ABI is follows the `extern "C"` calling convention, and is
//! 100% compatible with the [original C-based ABI](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html)
//! used in CS 452 at the University of Waterloo, insofar as the C-ABI
//! signatures and behaviors match (i.e: syscall numbers may vary between
//! different student implementations).

#![no_std]
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// A task identifier.
///
/// This is a FFI-safe newtype around `usize` that can only be constructed
/// through an unsafe constructor.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash, Deserialize, Serialize)]
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

/// Tid of the kernel-spawned name server.
///
/// This constant is the closure mechanism used by userspace tasks to know where
/// to begin name resolution.
///
/// While there's nothing stopping user tasks from directly sending messages to
/// the name server (using the `Send` syscall), this would be a _very bad idea_,
/// as the specific name server message protocol is not stable, and may change
/// at any time. As such, tasks should only use the provided name server API,
/// which is guaranteed to remain stable.
pub const NAMESERVER_TID: Tid = Tid(1);

/// Container for various bits of kernel performance data returned as part of
/// the `Perf` syscall.
// TODO: improve `struct PerfData`.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct PerfData {
    /// A number from 0 - 100
    pub idle_time_pct: u32,
}

/// Kernel syscall interface (i.e: syscall numbers, signatures, error codes)
pub mod syscall {
    /// Raw `choochoos` syscall numbers (used when invoking `swi #x`).
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

    /// Function signatures associated with each syscall. See the
    /// [CS 452 Website](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html)
    /// for more information of each of what these syscalls do.
    pub mod signature {
        #![allow(missing_docs)]

        use crate::{PerfData, Tid};

        pub type Yield = unsafe extern "C" fn();
        pub type Exit = unsafe extern "C" fn() -> !;
        pub type MyParentTid = unsafe extern "C" fn() -> isize;
        pub type MyTid = unsafe extern "C" fn() -> isize;
        pub type Create =
            unsafe extern "C" fn(priority: isize, function: Option<extern "C" fn() -> !>) -> isize;
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
        pub type AwaitEvent = unsafe extern "C" fn(event_id: usize) -> isize;

        /// Custom - Query kernel-specific [`PerfData`]
        pub type Perf = unsafe extern "C" fn(perf: *mut PerfData);
        /// Custom - Terminate the kernel.
        pub type Shutdown = unsafe extern "C" fn() -> !;
    }

    /// Errors returned by various syscalls.
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
        /// Note that message Truncation does _not_ correspond to an error code,
        /// and must be inferred by comparing the length of received message
        /// with the expected length.
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
        /// Note that message Truncation does _not_ correspond to an error code,
        /// and must be inferred by comparing the length of returned message
        /// with the expected length.
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum Reply {
            /// `tid` is not the task id of an existing task.
            TidDoesNotExist      = -1,
            /// `tid` is not the task id of a reply-blocked task.
            TidIsNotReplyBlocked = -2,
        }

        /// Errors returned by the `AwaitEvent` syscall.
        #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
        #[repr(isize)]
        pub enum AwaitEvent {
            /// Invalid event id.
            InvalidEventId        = -1,
            /// Corrupted volatile data.
            CorruptedVolatileData = -2,
        }
    }
}
