//! Syscall handler implementations. See the [`Kernel`] docs.

use abi::Tid;

use crate::util::user_slice::UserSlice;

use crate::kernel::task::TaskState;
use crate::kernel::{Kernel, ReadyQueueItem};

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_reply(
        &mut self,
        tid: Tid,
        reply: UserSlice<u8>,
    ) -> Result<usize, abi::syscall::error::Reply> {
        use abi::syscall::error::Reply as Error;

        let receiver = self
            .tasks
            .get_mut(tid.into())
            .ok_or(Error::TidDoesNotExist)?
            .as_mut()
            .ok_or(Error::TidDoesNotExist)?;

        let mut reply_dst = match receiver.state {
            TaskState::ReplyWait { reply_dst } => reply_dst,
            _ => return Err(Error::TidIsNotReplyBlocked),
        };

        let msg_len = reply_dst.copy_from_slice_min(reply);

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
}
