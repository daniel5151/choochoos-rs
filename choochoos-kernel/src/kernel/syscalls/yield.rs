//! Syscall handler implementations. See the [`Kernel`] docs.

use crate::kernel::Kernel;

/// Syscall handler implementations.
impl Kernel {
    pub fn syscall_yield(&mut self) {}
}
