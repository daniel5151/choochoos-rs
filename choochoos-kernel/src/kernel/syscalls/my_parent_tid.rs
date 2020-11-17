use abi::Tid;

use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_my_parent_tid(&mut self) -> Result<Tid, abi::syscall::error::MyParentTid> {
        let current_tid =
            (self.current_tid).expect("called exec_syscall while `current_tid == None`");

        self.tasks[current_tid.into()]
            .as_ref()
            .unwrap()
            .parent_tid
            .ok_or(abi::syscall::error::MyParentTid::NoParent)
    }
}
