//! Task descriptor data structures.

use core::ptr;

use abi::Tid;

use crate::util::user_slice::{UserSlice, UserSliceMut};

use super::arch::UserStack;

/// A Task's execution statue + state-specific associated data.
#[derive(Debug)]
pub enum TaskState {
    /// Ready (and waiting to be scheduled)
    Ready,
    /// Blocked - waiting to send a message.
    SendWait {
        /// The message being sent.
        msg_src: UserSlice<u8>,
        /// The reply buffer.
        reply_dst: UserSliceMut<u8>,
        /// Pointer to Sender waiting for this task to finish sending.
        next: Option<Tid>,
    },
    /// Blocked - waiting to receive a message.
    RecvWait {
        /// Where to write the Sender's Tid.
        sender_tid_dst: Option<ptr::NonNull<Tid>>,
        /// The receive buffer.
        recv_dst: UserSliceMut<u8>,
    },
    /// Blocked - waiting to receive a reply message.
    ReplyWait {
        /// The reply buffer.
        reply_dst: UserSliceMut<u8>,
    },
    /// Blocked - waiting for an event to occur.
    EventWait,
}

/// Task descriptor.
#[derive(Debug)]
pub struct TaskDescriptor {
    /// Scheduling priority (higher priority = preferential scheduling)
    pub priority: isize,
    /// Tid of parent task. The `FirstUserTask` and `NameServerTask` are spawned
    /// by the kernel, and have not parent task.
    pub parent_tid: Option<Tid>,
    /// A suspended task's stack pointer.
    pub sp: ptr::NonNull<UserStack>,

    /// The Tasks's execution state + state-specific associated data.
    pub state: TaskState,

    pub send_queue_head: Option<Tid>,
    pub send_queue_tail: Option<Tid>,
}

impl TaskDescriptor {
    /// Create a fresh `TaskDescriptor`.
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
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    ///
    /// # Panics
    ///
    /// Panics if the task is in the `Ready` state.
    pub fn inject_return_value<T: Copy>(&mut self, val: T) {
        if matches!(self.state, TaskState::Ready) {
            panic!("tried to inject return value while task was TaskState::Ready")
        }

        unsafe { self.sp.as_mut() }.inject_return_value(val)
    }
}
