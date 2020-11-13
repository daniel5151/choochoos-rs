//! Syscall handler implementations. See the [`Kernel`] docs.

use abi::Tid;

use crate::util::user_slice::{UserSlice, UserSliceMut};

use crate::kernel::task::TaskState;
use crate::kernel::{Kernel, ReadyQueueItem};

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_send(
        &mut self,
        receiver_tid: Tid,
        msg: UserSlice<u8>,
        reply: UserSliceMut<u8>,
    ) -> Result<(), abi::syscall::error::Send> {
        use abi::syscall::error::Send as Error;

        // ensure that the receiver exists
        if !self
            .tasks
            .get(receiver_tid.into())
            .map(Option::is_some)
            .unwrap_or(false)
        {
            return Err(Error::TidDoesNotExist);
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
                self.tasks[receiver_tid.into()].as_mut().unwrap()
            };
        }

        macro_rules! sender {
            () => {
                self.tasks[sender_tid.into()].as_mut().unwrap();
            };
        }

        let receiver = receiver!();
        match receiver.state {
            TaskState::RecvWait {
                sender_tid_dst,
                mut recv_dst,
            } => {
                let msg_len = recv_dst.copy_from_slice_min(msg);

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

                        let old_tail = self.tasks[receiver.send_queue_tail.unwrap().into()]
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
}
