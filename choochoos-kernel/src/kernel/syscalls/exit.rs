use crate::kernel::task::TaskState;
use crate::kernel::{Kernel, ReadyQueueItem};

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_exit(&mut self) {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");
        let task = self.tasks[current_tid.into()].as_mut().unwrap();

        // unblock any tasks that might be waiting for a response
        if let Some(mut tid) = task.send_queue_head {
            loop {
                let task = self.tasks[tid.into()].as_mut().unwrap();
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

        self.tasks[current_tid.into()] = None;
        self.current_tid = None;
    }
}
