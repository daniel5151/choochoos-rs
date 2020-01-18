use choochoos_sys::Tid;

use crate::TaskDescriptor;

pub struct Scheduler {
    tid_counter: usize,
    current_task: Option<TaskDescriptor>,
}

impl Scheduler {
    pub const fn new() -> Scheduler {
        Scheduler {
            tid_counter: 0,
            current_task: None,
        }
    }

    /// Returns a Tid to activate next, or None if there's nothing to schedule
    pub fn schedule(&mut self) -> Option<Tid> {
        self.current_task.as_ref().map(|td| td.tid)
    }

    fn get_free_tid(&mut self) -> Result<Tid, ()> {
        // TODO: recycle tids
        let ret = unsafe { Tid::from_raw(self.tid_counter) };
        self.tid_counter += 1;
        Ok(ret)
    }

    pub fn get_sp_mut(&mut self, tid: Tid) -> *mut usize {
        let _ = tid;
        self.current_task.as_ref().unwrap().sp
    }

    pub fn on_yield(&mut self, tid: Tid, new_sp: *mut usize) {
        let _ = tid;
        if let Some(td) = self.current_task.as_mut() {
            td.sp = new_sp
        }
    }

    /// Creates a new task, populating it's task descriptor, and returning a
    /// reference to the task's stack.
    ///
    /// The caller _must_ set up the stack correctly
    pub fn new_task(&mut self, priority: isize) -> (Tid, &mut *mut usize) {
        self.current_task = Some(TaskDescriptor {
            priority,
            tid: self.get_free_tid().unwrap(),
            parent_tid: None,
            sp: core::ptr::null_mut(),
        });

        let task = self.current_task.as_mut().unwrap();
        (task.tid, &mut task.sp)
    }

    pub fn exit_current_task(&mut self) {
        self.current_task = None;
    }
}
