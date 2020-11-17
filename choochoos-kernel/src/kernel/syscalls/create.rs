use abi::Tid;

use crate::kernel::task::TaskDescriptor;
use crate::kernel::{Kernel, ReadyQueueItem};

/// Syscall handler implementations.
impl Kernel {
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
            .map(|(i, _)| Tid::from(i))
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

            let start_of_stack = (&__USER_STACKS_START__ as *const _ as usize)
                + (USER_STACK_SIZE * (tid.into() + 1));

            crate::kernel::arch::fresh_stack(start_of_stack, function)
        };

        // create the new task descriptor
        self.tasks[tid.into()] = Some(TaskDescriptor::new(priority, self.current_tid, sp));

        self.ready_queue
            .push(ReadyQueueItem { tid, priority })
            .expect("out of space on the ready queue");

        Ok(tid)
    }
}
