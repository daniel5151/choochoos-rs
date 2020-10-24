use heapless::binary_heap::{BinaryHeap, Max};
use heapless::consts::*;

use abi::Tid;

mod arch;
mod syscalls;

pub mod task;

use task::{TaskDescriptor, TaskState};

/// Implements `Ord` by priority
#[derive(Debug, Eq, PartialEq)]
struct ReadyQueueItem {
    pub priority: isize,
    pub tid: Tid,
}

impl PartialOrd for ReadyQueueItem {
    fn partial_cmp(&self, rhs: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Ord for ReadyQueueItem {
    fn cmp(&self, rhs: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&rhs.priority)
    }
}

// oh const generics, please land soon
#[allow(non_camel_case_types)]
type MAX_TASKS = U16;
const MAX_TASKS: usize = 16;

/// The core choochoos kernel!
pub struct Kernel {
    tasks: [Option<TaskDescriptor>; MAX_TASKS],
    current_tid: Option<Tid>,
    ready_queue: BinaryHeap<ReadyQueueItem, MAX_TASKS, Max>, // matches number of tasks
}

impl Kernel {
    /// Set up the global kernel context.
    pub unsafe fn init() -> &'static mut Kernel {
        if let Some(ref mut kernel) = &mut crate::KERNEL {
            return kernel;
        }

        // Initialize the kernel's static state.
        let kernel = Kernel {
            tasks: Default::default(),
            current_tid: None,
            ready_queue: BinaryHeap::new(),
        };

        // Set the global kernel context.
        crate::KERNEL = Some(kernel);
        let kernel = crate::KERNEL.as_mut().unwrap();

        // perform any architecture specific kernel initialization
        // (e.g: registering interrupt/exception handlers)
        arch::init();

        // as a convenience, return a reference to the newly initialized global kernel
        kernel
    }

    /// Start the main Kernel loop.
    pub fn run(&mut self) {
        // spawn the first user task and the name server task.

        // provided by userspace
        #[link(name = "userspace", kind = "static")]
        extern "C" {
            fn FirstUserTask();
            fn NameServerTask();
        }

        // TODO: ensure that `NameServerTask` corresponds to abi::NAMESERVER_TID
        (self.syscall_create(0, Some(FirstUserTask))).unwrap();
        (self.syscall_create(0, Some(NameServerTask))).unwrap();

        // enter the main kernel loop
        loop {
            // determine which tid to schedule next
            let tid = match self.ready_queue.pop() {
                Some(item) => item.tid,
                // TODO: wait for IRQ
                None => return,
            };

            // activate the task
            self.current_tid = Some(tid);
            let sp = self.tasks[tid.raw()].as_mut().unwrap().sp;
            let next_sp = unsafe { arch::_activate_task(sp) };
            self.current_tid = None;

            // there's a chance that the task was exited / destroyed
            if let Some(ref mut task) = self.tasks[tid.raw()] {
                task.sp = next_sp;

                if matches!(task.state, TaskState::Ready) {
                    self.ready_queue
                        .push(ReadyQueueItem {
                            priority: task.priority,
                            tid,
                        })
                        .expect("out of space on the ready queue");
                }
            }
        }
    }

    fn get_free_tid(&mut self) -> Option<Tid> {
        // find first available none slot, and return a Tid corresponding to it's index
        self.tasks
            .iter()
            .enumerate()
            .find(|(_, t)| t.is_none())
            .map(|(i, _)| unsafe { Tid::from_raw(i) })
    }

    /// Utility method to retrieve the current_tid. Only used when the `kdebug`
    /// feature is active.
    #[doc(hidden)]
    #[allow(dead_code)]
    #[cfg(feature = "kdebug")]
    pub(crate) fn current_tid(&self) -> Option<Tid> {
        self.current_tid
    }
}
