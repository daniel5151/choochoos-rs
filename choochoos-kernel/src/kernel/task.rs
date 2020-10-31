use core::ptr;

use abi::Tid;

use crate::util::user_slice::{UserSlice, UserSliceMut};

use super::arch::UserStack;

#[derive(Debug)]
pub enum TaskState {
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
    EventWait,
}

pub struct TaskDescriptor {
    pub priority: isize,
    pub parent_tid: Option<Tid>,
    pub sp: ptr::NonNull<UserStack>,

    pub state: TaskState,

    pub send_queue_head: Option<Tid>,
    pub send_queue_tail: Option<Tid>,
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
