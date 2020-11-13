//! Syscall handler implementations. See the [`Kernel`] docs.

use abi::Tid;

use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_my_tid(&mut self) -> Tid {
        (self.current_tid).expect("called exec_syscall while `current_tid == None`")
    }
}
