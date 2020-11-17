use core::ptr;

use abi::Tid;

use crate::util::user_slice::UserSliceMut;

use crate::kernel::task::TaskState;
use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_receive(
        &mut self,
        sender_tid_dst: Option<ptr::NonNull<Tid>>,
        mut msg_dst: UserSliceMut<u8>,
    ) -> Option<usize> {
        let receiver_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let receiver = self.tasks[receiver_tid.into()].as_mut().unwrap();

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
                // return value written later, as part of the send syscall
                return None;
            }
        };

        let sender = self.tasks[sender_tid.into()]
            .as_mut()
            .expect("sender was unexpectedly missing");

        let msg_src = match sender.state {
            TaskState::SendWait { msg_src, .. } => msg_src,
            _ => panic!("sender was not in SendWait state"),
        };

        let msg_len = msg_dst.copy_from_slice_min(msg_src);

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

        let receiver = self.tasks[receiver_tid.into()].as_mut().unwrap();

        receiver.send_queue_head = next;
        match receiver.send_queue_head {
            None => receiver.send_queue_tail = None,
            Some(_) => assert!(receiver.send_queue_tail.is_some()),
        }

        Some(msg_len)
    }
}
