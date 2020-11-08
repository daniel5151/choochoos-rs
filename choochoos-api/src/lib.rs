//! An idiomatic Rust API on-top of the raw `choochoos` ABI.

#![no_std]
#![deny(missing_docs)]
#![feature(asm, naked_functions)]

use core::num::NonZeroUsize;

pub use abi;
pub use abi::Tid;

#[allow(non_snake_case, unused_variables)]
mod raw {
    use super::abi::{PerfData, Tid};

    macro_rules! raw_syscall {
        ($no:literal => $($sig:tt)*) => {
            #[naked]
            #[inline(never)]
            pub unsafe extern "C" $($sig)* {
                asm! {
                    concat!("swi #", stringify!($no)),
                    "bx lr",
                }
                unreachable!()
            }
        };
    }

    raw_syscall!(0 => fn __Yield());
    raw_syscall!(1 => fn __Exit());
    raw_syscall!(2 => fn __MyTid() -> isize);
    raw_syscall!(3 => fn __MyParentTid() -> isize);
    raw_syscall!(4 => fn __Create(priority: isize, function: Option<extern "C" fn()>) -> isize);
    raw_syscall!(5 => fn __Send(
            tid: Tid,
            msg: *const u8,
            msglen: usize,
            reply: *mut u8,
            rplen: usize,
        ) -> isize);
    raw_syscall!(6 => fn __Receive(tid: *mut Tid, msg: *mut u8, msglen: usize) -> isize);
    raw_syscall!(7 => fn __Reply(tid: Tid, reply: *const u8, rplen: usize) -> isize);
    raw_syscall!(8 => fn __AwaitEvent(event_id: usize) -> isize);

    raw_syscall!(9 => fn __Perf(perf: *mut PerfData));

    // Ensure that the type signatures match those defined in `choochoos_abi`
    mod abi_assert {
        use super::*;
        use abi::syscall::signature as sig;

        const _: sig::Yield = __Yield;
        const _: sig::Exit = __Exit;
        const _: sig::MyTid = __MyTid;
        const _: sig::MyParentTid = __MyParentTid;
        const _: sig::Create = __Create;
        const _: sig::Send = __Send;
        const _: sig::Receive = __Receive;
        const _: sig::Reply = __Reply;
        const _: sig::AwaitEvent = __AwaitEvent;

        const _: sig::Perf = __Perf;
    }
}

/// Errors which may occur when invoking syscalls.
pub mod error {
    use core::num::NonZeroUsize;

    /// Errors returned by the `Create` syscall.
    #[derive(Debug)]
    pub enum Create {
        /// Tried to create a task with an invalid priority.
        InvalidPriority,
        /// The Kernel has run out of task descriptors.
        OutOfTaskDescriptors,
    }

    /// Errors returned by the `Send` syscall.
    #[derive(Debug)]
    pub enum Send {
        /// `tid` is not the task id of an existing task.
        TidDoesNotExist,
        /// The send-receive-reply transaction could not be completed.
        CouldNotSSR,
        /// The reply was truncated. `usize` corresponds to the length of the
        /// original reply.
        Truncated(NonZeroUsize),
    }

    /// Errors returned by the `Receive` syscall.
    #[derive(Debug)]
    pub enum Receive {
        /// The message was truncated. `usize` corresponds to the length of the
        /// original message.
        Truncated(NonZeroUsize),
    }

    /// Errors returned by the `Reply` syscall.
    #[derive(Debug)]
    pub enum Reply {
        /// `tid` is not the task id of an existing task.
        TidDoesNotExist,
        /// `tid` is not the task id of a reply-blocked task.
        TidIsNotReplyBlocked,
        /// The reply was truncated. `usize` corresponds to the number of bytes
        /// successfully written.
        Truncated(NonZeroUsize),
    }

    /// Errors returned by the `AwaitEvent` syscall.
    #[derive(Debug)]
    pub enum AwaitEvent {
        /// Invalid event id.
        InvalidEventId,
        /// Corrupted volatile data.
        CorruptedVolatileData,
    }
}

/// Causes a task to pause executing.
/// The task is moved to the end of its priority queue, and will resume
/// executing when next scheduled.
pub fn r#yield() {
    unsafe { raw::__Yield() }
}

/// Causes a task to cease execution permanently. It is removed from all
/// priority queues, send queues, receive queues and event queues. Resources
/// owned by the task, primarily its memory and task descriptor, are not
/// reclaimed.
///
/// NOTE: Each task must call `exit` when it returns!
pub fn exit() {
    unsafe { raw::__Exit() }
}

/// Returns the task id of the calling task.
pub fn my_tid() -> Tid {
    // SAFETY: The MyTid syscall cannot return an error
    unsafe { Tid::from_raw(raw::__MyTid() as usize) }
}

/// Returns the task id of the task that created the calling task.
///
/// Returns [`None`] if the parent task has exited or been destroyed.
pub fn my_parent_tid() -> Option<Tid> {
    let ret = unsafe { raw::__MyParentTid() };
    match ret {
        e if e < 0 => None,
        // SAFETY: tid is guaranteed to be greater than zero
        tid => Some(unsafe { Tid::from_raw(tid as usize) }),
    }
}

/// Allocates and initializes a task descriptor, using the given priority,
/// and the given function pointer as a pointer to the entry point of
/// executable code.
///
/// If `create` returns successfully, the task descriptor has all the state
/// needed to run the task, the task’s stack has been suitably initialized, and
/// the task has been entered into its ready queue so that it will run the next
/// time it is scheduled.
pub fn create(priority: usize, function: extern "C" fn()) -> Result<Tid, error::Create> {
    let ret = unsafe { raw::__Create(priority as isize, Some(function)) };
    match ret {
        e if ret < 0 => match e {
            -1 => Err(error::Create::InvalidPriority),
            -2 => Err(error::Create::OutOfTaskDescriptors),
            _ => panic!("unexpected Create error: {}", e),
        },
        // SAFETY: tid is guaranteed to be greater than zero
        tid => Ok(unsafe { Tid::from_raw(tid as usize) }),
    }
}

/// Sends a message to another task and receives a reply.
///
/// The message, in a buffer in the sending task’s memory, is copied to the
/// memory of the task to which it is sent by the kernel. `send()` supplies a
/// buffer into which the reply is to be copied, and the size of the reply
/// buffer, so that the kernel can detect overflow.
///
/// When `send()` returns without error it is guaranteed that the message has
/// been received, and that a reply has been sent, not necessarily by the same
/// task.
///
/// The kernel will not overflow the reply buffer. If the size of the
/// reply set exceeds the length of the reply buffer, the reply is truncated
/// and a [`error::Send::Truncated`] is returned.
///
/// There is no guarantee that `send()` will return. If, for example, the task
/// to which the message is directed never calls `receive()`, `send()` never
/// returns and the sending task remains blocked forever.
pub fn send(tid: Tid, msg: &[u8], reply: &mut [u8]) -> Result<usize, error::Send> {
    let ret = unsafe {
        raw::__Send(
            tid,
            msg.as_ptr(),
            msg.len(),
            reply.as_mut_ptr(),
            reply.len(),
        )
    };
    match ret {
        e if ret < 0 => match e {
            -1 => Err(error::Send::TidDoesNotExist),
            -2 => Err(error::Send::CouldNotSSR),
            _ => panic!("unexpected Send error: {}", e),
        },
        rplen => {
            let rplen = rplen as usize;
            if rplen > reply.len() {
                // SAFETY: if rplen was zero, then `0 > reply.len(): usize` would never trigger
                let rplen = unsafe { NonZeroUsize::new_unchecked(rplen) };
                Err(error::Send::Truncated(rplen))
            } else {
                Ok(rplen)
            }
        }
    }
}

/// Blocks until a message is sent to the caller, returning the Tid of the task
/// that sent the message and the number of bytes in the message.
///
/// Messages sent before `receive()` is called are retained in a send queue,
/// from which they are received in first-come, first-served order.
///
/// The kernel will not overflow the message buffer. If the size of the message
/// set exceeds msglen, the message is truncated and the buffer contains the
/// and a [`error::Receive::Truncated`] is returned.
pub fn receive(msg: &mut [u8]) -> Result<(Tid, usize), error::Receive> {
    let mut tid = unsafe { Tid::from_raw(0) };
    let ret = unsafe { raw::__Receive(&mut tid, msg.as_mut_ptr(), msg.len()) };
    match ret {
        e if ret < 0 => panic!("unexpected Receive error: {}", e),
        msglen => {
            let msglen = msglen as usize;
            if msglen > msg.len() {
                // SAFETY: if msglen was zero, then `0 > msg.len(): usize` would never trigger
                let msglen = unsafe { NonZeroUsize::new_unchecked(msglen) };
                Err(error::Receive::Truncated(msglen))
            } else {
                Ok((tid, msglen as usize))
            }
        }
    }
}

/// Sends a reply to a task that previously sent a message.
///
/// When it returns without error, the reply has been entirely copied into the
/// sender’s memory. If the message was truncated, as
/// [`error::Reply::Truncated`] is returned.
///
/// The calling task and the sender return at the same logical time, so
/// whichever is of higher priority runs first. If they are of the same
/// priority, the sender runs first.
pub fn reply(tid: Tid, reply: &[u8]) -> Result<usize, error::Reply> {
    let ret = unsafe { raw::__Reply(tid, reply.as_ptr(), reply.len()) };
    match ret {
        e if ret < 0 => match e {
            -1 => Err(error::Reply::TidDoesNotExist),
            -2 => Err(error::Reply::TidIsNotReplyBlocked),
            _ => panic!("unexpected Reply error: {}", e),
        },
        rplen => {
            let rplen = rplen as usize;
            if rplen < reply.len() {
                // SAFETY: if rplen was zero, then `0 > reply.len(): usize` would never trigger
                let rplen = unsafe { NonZeroUsize::new_unchecked(rplen) };
                Err(error::Reply::Truncated(rplen))
            } else {
                Ok(rplen)
            }
        }
    }
}

/// Blocks until the event identified by event_id occurs, then returns with
/// volatile data (if applicable).
///
/// Valid `event_id` numbers vary based on target platform.
pub fn await_event(event_id: usize) -> Result<usize, error::AwaitEvent> {
    let ret = unsafe { raw::__AwaitEvent(event_id) };
    match ret {
        e if ret < 0 => match e {
            -1 => Err(error::AwaitEvent::InvalidEventId),
            -2 => Err(error::AwaitEvent::CorruptedVolatileData),
            _ => panic!("unexpected AwaitEvent error: {}", e),
        },
        volatile => Ok(volatile as usize),
    }
}

/// (custom ext) Return Kernel performance data (e.g: idle time).
pub fn perf() -> abi::PerfData {
    unsafe {
        let mut perf_data = core::mem::MaybeUninit::<abi::PerfData>::uninit();
        raw::__Perf(perf_data.as_mut_ptr());
        perf_data.assume_init()
    }
}
