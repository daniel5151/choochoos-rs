//! Syscall handler implementations. See the [`Kernel`] docs.

use core::ptr;

use abi::Tid;

use crate::util::user_slice::{UserSlice, UserSliceMut};

use super::task::{TaskDescriptor, TaskState};
use super::{EventQueueItem, Kernel, ReadyQueueItem};

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_yield(&mut self) {}

    pub fn syscall_exit(&mut self) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let task = self.tasks[current_tid.raw()].as_mut().unwrap();

        // unblock any tasks that might be waiting for a response
        if let Some(mut tid) = task.send_queue_head {
            loop {
                let task = self.tasks[tid.raw()].as_mut().unwrap();
                let next_tid = match task.state {
                    TaskState::SendWait { next, .. } => next,
                    _ => panic!(),
                };

                // SRR could not be completed, return -2 to the sender
                task.inject_return_value(abi::syscall::error::Send::CouldNotSSR);
                task.state = TaskState::Ready;
                self.ready_queue
                    .push(ReadyQueueItem {
                        tid,
                        priority: task.priority,
                    })
                    .expect("out of space on the ready queue");

                match next_tid {
                    None => break,
                    Some(next_tid) => tid = next_tid,
                }
            }
        }

        self.tasks[current_tid.raw()] = None;
        self.current_tid = None;
    }

    pub fn syscall_my_tid(&mut self) -> Tid {
        (self.current_tid).expect("called exec_syscall while `current_tid == None`")
    }

    pub fn syscall_my_parent_tid(&mut self) -> Result<Tid, abi::syscall::error::MyParentTid> {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        self.tasks[current_tid.raw()]
            .as_ref()
            .unwrap()
            .parent_tid
            .ok_or(abi::syscall::error::MyParentTid::NoParent)
    }

    pub fn syscall_create(
        &mut self,
        priority: isize,
        function: Option<unsafe extern "C" fn()>,
    ) -> Result<Tid, abi::syscall::error::Create> {
        use abi::syscall::error::Create as Error;

        let function = match function {
            Some(f) => f,
            // TODO? make this an error code?
            None => panic!("Cannot create task with null pointer"),
        };

        // this is an artificial limitation, but it could come in handy in the future.
        if priority < 0 {
            return Err(Error::InvalidPriority);
        }

        // find first available none slot
        let tid = self
            .tasks
            .iter()
            .enumerate()
            .find(|(_, t)| t.is_none())
            .map(|(i, _)| unsafe { Tid::from_raw(i) })
            .ok_or(Error::OutOfTaskDescriptors)?;

        // set up a fresh stack for the new task. This requires some unsafe,
        // arch-specific, low-level shenanigans.
        // TODO: this should be platform specific code...
        let sp = unsafe {
            // provided by the linker
            extern "C" {
                static __USER_STACKS_START__: core::ffi::c_void;
            }

            // TODO: find a smarter user stack size number
            const USER_STACK_SIZE: usize = 0x40000;

            let start_of_stack =
                (&__USER_STACKS_START__ as *const _ as usize) + (USER_STACK_SIZE * (tid.raw() + 1));

            super::arch::fresh_stack(start_of_stack, function)
        };

        // create the new task descriptor
        self.tasks[tid.raw()] = Some(TaskDescriptor::new(priority, self.current_tid, sp));

        self.ready_queue
            .push(ReadyQueueItem { tid, priority })
            .expect("out of space on the ready queue");

        Ok(tid)
    }

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
            .get(receiver_tid.raw())
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
                self.tasks[receiver_tid.raw()].as_mut().unwrap()
            };
        }

        macro_rules! sender {
            () => {
                self.tasks[sender_tid.raw()].as_mut().unwrap();
            };
        }

        let receiver = receiver!();
        match receiver.state {
            TaskState::RecvWait {
                sender_tid_dst,
                mut recv_dst,
            } => {
                let msg_len = msg.len().min(recv_dst.len());
                recv_dst[..msg_len].copy_from_slice(&msg[..msg_len]);

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

                        let old_tail = self.tasks[receiver.send_queue_tail.unwrap().raw()]
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

    pub fn syscall_recieve(
        &mut self,
        sender_tid_dst: Option<ptr::NonNull<Tid>>,
        mut msg_dst: UserSliceMut<u8>,
    ) -> Option<usize> {
        let receiver_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let receiver = self.tasks[receiver_tid.raw()].as_mut().unwrap();

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

        let sender = self.tasks[sender_tid.raw()]
            .as_mut()
            .expect("sender was unexpectedly missing");

        let msg_src = match sender.state {
            TaskState::SendWait { msg_src, .. } => msg_src,
            _ => panic!("sender was not in SendWait state"),
        };

        let msg_len = msg_src.len().min(msg_dst.len());

        msg_dst[..msg_len].copy_from_slice(&msg_src[..msg_len]);

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

        let receiver = self.tasks[receiver_tid.raw()].as_mut().unwrap();

        receiver.send_queue_head = next;
        match receiver.send_queue_head {
            None => receiver.send_queue_tail = None,
            Some(_) => assert!(receiver.send_queue_tail.is_some()),
        }

        Some(msg_len)
    }

    pub fn syscall_reply(
        &mut self,
        tid: Tid,
        reply: UserSlice<u8>,
    ) -> Result<usize, abi::syscall::error::Reply> {
        use abi::syscall::error::Reply as Error;

        let receiver = self
            .tasks
            .get_mut(tid.raw())
            .ok_or(Error::TidDoesNotExist)?
            .as_mut()
            .ok_or(Error::TidDoesNotExist)?;

        let mut reply_dst = match receiver.state {
            TaskState::ReplyWait { reply_dst } => reply_dst,
            _ => return Err(Error::TidIsNotReplyBlocked),
        };

        let msg_len = reply_dst.len().min(reply.len());

        reply_dst[..msg_len].copy_from_slice(&reply[..msg_len]);

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

    pub fn syscall_await_event(
        &mut self,
        event_id: usize,
    ) -> Result<Option<usize>, abi::syscall::error::AwaitEvent> {
        use abi::syscall::error::AwaitEvent as Error;

        if !crate::platform::interrupts::validate_eventid(event_id) {
            return Err(Error::InvalidEventId);
        }

        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let task = self.tasks[current_tid.raw()].as_mut().unwrap();

        if let Some(tid_or_volatile_data) = self.event_queue.remove(&event_id) {
            match tid_or_volatile_data {
                EventQueueItem::BlockedTid(tid) => {
                    // TODO: support multiple tasks waiting on the same event?
                    panic!(
                        "AwaitEvent({}): {:?} is already waiting for this event",
                        event_id, tid
                    );
                }
                EventQueueItem::VolatileData(data) => {
                    kdebug!(
                        "AwaitEvent({}): data already arrived {:#x?}",
                        event_id,
                        data
                    );

                    return Ok(Some(data));
                }
            }
        }

        kdebug!(
            "AwaitEvent({}): put {:?} on event_queue",
            event_id,
            current_tid
        );
        self.event_queue
            .insert(event_id, EventQueueItem::BlockedTid(current_tid))
            .expect("out of space on the event queue");

        assert!(matches!(task.state, TaskState::Ready));
        task.state = TaskState::EventWait;

        Ok(None)
    }

    pub fn syscall_perf(&mut self, perf_data: Option<ptr::NonNull<abi::PerfData>>) {
        if let Some(mut perf_data) = perf_data {
            let perf_data = unsafe { perf_data.as_mut() };

            perf_data.idle_time_pct = 0; // XXX: actually track idle time
        }
    }
}
