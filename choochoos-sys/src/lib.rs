#![no_std]

/// Function signature which can be spawned by the kernel
pub type TaskFn = extern "C" fn();

/// Task descriptor handle
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Tid(usize);

impl Tid {
    /// Create a new Tid from a raw value
    ///
    /// # Safety
    ///
    /// The val should always correspond to a valid task descriptor.
    pub unsafe fn from_raw(val: usize) -> Tid {
        Tid(val)
    }

    /// Return the Tid's raw value
    pub fn raw(self) -> usize {
        self.0
    }
}

pub mod error {
    /// Errors returned by the `Create` syscall
    #[derive(Debug)]
    pub enum CreateError {
        InvalidPriority,
        OutOfTaskDescriptors,
    }
}

mod raw {
    extern "C" {
        pub fn Yield();
        pub fn Exit();
        pub fn MyTid() -> isize;
        pub fn MyParentTid() -> isize;
        pub fn Create(priority: isize, function: Option<extern "C" fn()>) -> isize;
    }
}

/// Causes a task to pause executing.
/// The task is moved to the end of its priority queue, and will resume
/// executing when next scheduled.
pub fn r#yield() {
    unsafe { raw::Yield() }
}

/// Causes a task to cease execution permanently. It is removed from all
/// priority queues, send queues, receive queues and event queues. Resources
/// owned by the task, primarily its memory and task descriptor, are not
/// reclaimed.
///
/// NOTE: Each task implicitly calls `exit` when it returns!
pub fn exit() {
    unsafe { raw::Exit() }
}

/// Returns the task id of the calling task.
pub fn my_tid() -> Tid {
    let raw_mytid_retval = unsafe { raw::MyTid() };
    // TODO: assert that raw_tid >= 0
    Tid(raw_mytid_retval as usize)
}

/// Returns the task id of the task that created the calling task.
///
/// Returns [`None`] if the parent task has exited or been destroyed.
pub fn my_parent_tid() -> Option<Tid> {
    let raw_parent_tid_retval = unsafe { raw::MyParentTid() };
    if raw_parent_tid_retval < 0 {
        None
    } else {
        Some(Tid(raw_parent_tid_retval as usize))
    }
}

/// Allocates and initializes a task descriptor, using the given priority,
/// and the given function pointer as a pointer to the entry point of
/// executable code.
///
/// If create returns successfully, the task descriptor has all the state
/// needed to run the task, the task’s stack has been suitably
/// initialized, and the task has been entered into its ready queue so
/// that it will run the next time it is scheduled.
pub fn create(priority: isize, function: extern "C" fn()) -> Result<Tid, error::CreateError> {
    let raw_create_retval = unsafe { raw::Create(priority, Some(function)) };
    match raw_create_retval {
        -1 => Err(error::CreateError::InvalidPriority),
        -2 => Err(error::CreateError::OutOfTaskDescriptors),
        tid => {
            if tid < 0 {
                // TODO: assert this can should never happen
                unreachable!()
            } else {
                Ok(Tid(tid as usize))
            }
        }
    }
}
