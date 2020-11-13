//! Syscall handler implementations. See the [`Kernel`] docs.

use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_shutdown(&mut self) {
        self.event_queue.clear();
        self.ready_queue.clear();
        self.tasks.iter_mut().for_each(|t| *t = None);
        self.current_tid = None;
    }
}
