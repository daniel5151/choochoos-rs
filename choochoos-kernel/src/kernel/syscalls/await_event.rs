use crate::kernel::task::TaskState;
use crate::kernel::{EventQueueItem, Kernel};

/// Syscall handler implementations.
impl Kernel {
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
        let task = self.tasks[current_tid.into()].as_mut().unwrap();

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
}
