use heapless::binary_heap::{BinaryHeap, Max};
use heapless::consts::*;
use heapless::LinearMap;

use abi::Tid;

mod arch;
mod syscalls;

pub mod task;

use task::{TaskDescriptor, TaskState};

/// A pair of `Tid` and it's `priority`.
///
/// Implements [`Ord`] using the `priority` field.
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

/// Either the Tid of a Task waiting for an Event, or some VolatileData from an
/// interrupt that no Task was waiting for.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum EventQueueItem {
    BlockedTid(Tid),
    VolatileData(usize),
}

// oh const generics, please land soon
#[allow(non_camel_case_types)]
type MAX_TASKS = U16;
const MAX_TASKS: usize = 16;
#[allow(non_camel_case_types)]
type MAX_EVENTS = U16;

/// The core choochoos kernel!
pub struct Kernel {
    /// Fixed-size array of TaskDescriptor.
    ///
    /// At the moment, [`Tid`]s are direct indexes into this array.
    tasks: [Option<TaskDescriptor>; MAX_TASKS],
    /// The currently running Tid.
    current_tid: Option<Tid>,
    /// Priority queue of tasks ready to be scheduled.
    ready_queue: BinaryHeap<ReadyQueueItem, MAX_TASKS, Max>, // matches number of tasks
    /// A map of `event_id`s to either a blocked task, or some unclaimed
    /// volatile data.
    event_queue: LinearMap<usize, EventQueueItem, MAX_EVENTS>,
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
            event_queue: LinearMap::new(),
        };

        // Set the global kernel context.
        crate::KERNEL = Some(kernel);
        let kernel = crate::KERNEL.as_mut().unwrap();

        // perform any architecture specific initialization
        // (e.g: registering interrupt/exception handlers)
        arch::init();

        // perform any platform specific initialization
        // (e.g: activating device interrupts)
        crate::platform::initialize();

        // as a convenience, return a reference to the newly initialized global kernel
        kernel
    }

    /// Start the main Kernel loop.
    pub fn run(&mut self) {
        // spawn the first user task and the name server task.

        // provided by userspace
        extern "C" {
            fn FirstUserTask();
            fn NameServerTask();
        }

        self.syscall_create(0, Some(FirstUserTask)).unwrap();
        let ns_tid = self.syscall_create(0, Some(NameServerTask)).unwrap();

        assert_eq!(ns_tid, abi::NAMESERVER_TID);

        // enter the main kernel loop
        loop {
            // determine which tid to schedule next
            let tid = match self.ready_queue.pop() {
                Some(item) => item.tid,
                None => {
                    if self.event_queue.is_empty() {
                        break;
                    }

                    // TODO: track idle time
                    let _time_asleep = unsafe { crate::platform::idle_task() };
                    continue;
                }
            };

            // activate the task
            self.current_tid = Some(tid);
            let sp = self.tasks[tid.into()].as_mut().unwrap().sp;
            let next_sp = unsafe { arch::_activate_task(sp) };
            self.current_tid = None;

            // there's a chance that the task was exited / destroyed
            if let Some(ref mut task) = self.tasks[tid.into()] {
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

        unsafe { crate::platform::teardown() };
    }

    /// Utility method to retrieve the current_tid. Only used when the `kdebug`
    /// feature is active.
    #[doc(hidden)]
    #[allow(dead_code)]
    #[cfg(feature = "kdebug")]
    pub(crate) fn current_tid(&self) -> Option<Tid> {
        self.current_tid
    }

    pub unsafe fn handle_irq(&mut self) {
        crate::platform::interrupts::handle_irq(|event_id: usize, volatile_data: usize| {
            match self.event_queue.remove(&event_id) {
                None => {
                    kdebug!(
                        "no tasks are waiting for event_id {}, storing data {:#x?}",
                        event_id,
                        volatile_data
                    );
                    self.event_queue
                        .insert(event_id, EventQueueItem::VolatileData(volatile_data))
                        .expect("no more room in event_queue");
                }
                Some(EventQueueItem::BlockedTid(tid)) => {
                    let blocked_task = match self.tasks[tid.into()] {
                        Some(ref mut task) => task,
                        None => {
                            kdebug!(
                                "{:?} was terminated while waiting for event_id {}",
                                tid,
                                event_id
                            );
                            return;
                        }
                    };
                    assert!(matches!(blocked_task.state, TaskState::EventWait));
                    blocked_task.inject_return_value(volatile_data);
                    blocked_task.state = TaskState::Ready;

                    self.ready_queue
                        .push(ReadyQueueItem {
                            tid,
                            priority: blocked_task.priority,
                        })
                        .expect("no more space on ready_queue");
                }
                Some(EventQueueItem::VolatileData(_old_data)) => {
                    self.event_queue
                        .insert(event_id, EventQueueItem::VolatileData(volatile_data))
                        .unwrap(); // removed an item, so there must be room
                }
            }
        });
    }
}
