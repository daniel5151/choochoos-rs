use choochoos_sys::Tid;

mod mailbox {
    generic_containers::impl_queue!(16);
}

use mailbox::Queue as Mailbox;

#[derive(Debug)]
pub enum TaskState<'a> {
    Ready { mailbox: Mailbox<&'a [u8]> },
    ReceiveWait { recv_buf: &'a mut [u8] },
    ReplyWait { reply_buf: &'a mut [u8] },
}

pub struct TaskDescriptor<'a> {
    priority: isize,
    tid: Tid,
    parent_tid: Option<Tid>,
    sp: *mut usize,
    state: TaskState<'a>,
}

impl<'a> TaskDescriptor<'a> {
    pub fn is_ready(&self) -> bool {
        match self.state {
            TaskState::Ready { .. } => true,
            _ => false,
        }
    }
}

mod pq {
    generic_containers::impl_priority_queue!(8, 16);
}
use pq::PriorityQueue;

pub struct Scheduler {
    tasks: [Option<TaskDescriptor<'static>>; 8],
    current_tid: Option<Tid>,
    ready_queue: PriorityQueue<Tid>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: PriorityQueue::new(),
        }
    }

    /// Returns a Tid to activate next, or None if there's nothing to schedule
    pub fn schedule(&mut self) -> Option<Tid> {
        match self.ready_queue.pop() {
            Err(_) => None,
            Ok(tid) => match &self.tasks[tid.raw()].as_ref() {
                None => panic!("trying to schedule inactive task tid={:?}", tid),
                Some(task) => {
                    match &task.state {
                        TaskState::Ready { .. } => (),
                        state => panic!(
                            "trying to schedule task in non-ready state: tid={:?} state={:?}",
                            tid, state
                        ),
                    }
                    self.current_tid = Some(tid);
                    Some(tid)
                }
            },
        }
    }

    /// Returns a free Tid
    fn get_free_tid(&mut self) -> Option<Tid> {
        // find first available none slot, and return a Tid corresponding to it's index
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, t)| t.is_none())
            .map(|(i, _)| unsafe { Tid::from_raw(i) })
    }

    pub fn get_sp_mut(&mut self, tid: Tid) -> *mut usize {
        self.tasks[tid.raw()].as_mut().unwrap().sp
    }

    pub fn on_yield(&mut self, tid: Tid, new_sp: *mut usize) {
        match self.tasks[tid.raw()].as_mut() {
            None => {
                // task was exited / destroyed
            }
            Some(task) => {
                task.sp = new_sp;
                if task.is_ready() {
                    self.ready_queue
                        .push(tid, task.priority as usize)
                        .expect("out of space on the ready queue")
                }
            }
        }
    }

    pub fn handle_send(&mut self, tid: Tid, msg: &'static [u8]) -> isize {
        match self.tasks[tid.raw()].as_mut() {
            None => -1,
            Some(receiver) => {
                use TaskState::*;
                match &mut receiver.state {
                    Ready { mailbox } => match mailbox.push_back(msg) {
                        Ok(()) => 0,
                        Err(_) => -2, // mailbox was full
                    },
                    ReplyWait { reply_buf } => -2,
                    ReceiveWait { recv_buf } => {
                        let l = core::cmp::min(msg.len(), recv_buf.len());
                        recv_buf[..l].clone_from_slice(&msg[..l]);
                        receiver.state = Ready {
                            mailbox: Mailbox::new(),
                        };
                        self.ready_queue
                            .push(tid, receiver.priority as usize)
                            .expect("out of space on the ready queue");
                        0
                    }
                }
            }
        }
    }
    pub fn handle_receive(&mut self, tid: &mut Tid, recv_buf: &'static mut [u8]) -> isize {
        use TaskState::*;
        let tid = self.current_tid.unwrap();
        let task = self.tasks[tid.raw()]
            .as_mut()
            .expect("Receive() from an inactive task");

        match &mut task.state {
            ReplyWait { .. } | ReceiveWait { .. } => panic!(
                "Receive() from a task in an invalid state: {:?}",
                task.state
            ),
            Ready { mailbox } => match mailbox.pop_front() {
                Err(_) => {
                    task.state = ReceiveWait { recv_buf };
                    0
                }
                Ok(msg) => {
                    let l = core::cmp::min(msg.len(), recv_buf.len());
                    recv_buf[..l].clone_from_slice(&msg[..l]);
                    task.state = Ready {
                        mailbox: Mailbox::new(),
                    };
                    self.ready_queue
                        .push(tid, task.priority as usize)
                        .expect("out of space on the ready queue");
                    0
                }
            },
        }
    }
    pub fn handle_reply(&mut self, tid: Tid, reply: &[u8]) -> isize {
        unimplemented!()
    }

    /// Create a new task with the given priority. Returns None if the kernel
    /// has run out of `Tid`s.
    ///
    /// The caller is required to provide a stack initiazation routine, which
    /// returns a pointer to a freshly set up and otherwise unused stack.
    ///
    /// # Safety
    ///
    /// If the stack_init_fn sets up the stack incorrectly, switching to the
    /// task will result in undefined behavior.
    pub unsafe fn new_task(
        &mut self,
        priority: isize,
        stack_init_fn: impl FnOnce(Tid) -> *mut usize,
    ) -> Option<Tid> {
        let tid = self.get_free_tid()?;

        self.tasks[tid.raw()] = Some(TaskDescriptor {
            priority,
            tid,
            parent_tid: self.current_tid(),
            sp: stack_init_fn(tid),
            state: TaskState::Ready {
                mailbox: Mailbox::new(),
            },
        });

        self.ready_queue
            .push(tid, priority as usize)
            .expect("out of space on the ready queue");

        Some(tid)
    }

    /* ------------ Syscall Helpers ------------ */

    /// Exits the currently running task
    pub fn exit_current_task(&mut self) {
        self.tasks[self.current_tid.unwrap().raw()] = None;
        self.current_tid = None;
    }

    pub fn current_tid(&self) -> Option<Tid> {
        self.current_tid
    }

    pub fn current_parent_tid(&self) -> Option<Tid> {
        self.tasks[self.current_tid?.raw()]
            .as_ref()
            .map(|t| t.parent_tid)
            .flatten()
    }
}
