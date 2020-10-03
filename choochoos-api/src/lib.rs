//! An idiomatic Rust API on-top of raw `choochoos` syscalls.

#![no_std]

pub use choochoos_abi as abi;

use abi::Tid;

pub mod error {
    /// Errors returned by the `Create` syscall
    #[derive(Debug)]
    pub enum Create {
        InvalidPriority,
        OutOfTaskDescriptors,
    }
}

mod raw {
    extern "C" {
        pub fn __Yield();
        pub fn __Exit();
        pub fn __MyTid() -> isize;
        pub fn __MyParentTid() -> isize;
        pub fn __Create(priority: isize, function: Option<extern "C" fn()>) -> isize;
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
/// NOTE: Each task implicitly calls `exit` when it returns!
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
            _ => panic!("unexpected create error: {}", e),
        },
        // SAFETY: tid is guaranteed to be greater than zero
        tid => Ok(unsafe { Tid::from_raw(tid as usize) }),
    }
}
