//! Types and structures exposed by the `choochoos` kernel.

#![no_std]

/// Function signature which can be spawned by the kernel.
pub type TaskFn = extern "C" fn();

/// Task descriptor handle.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Tid(usize);

impl Tid {
    /// Create a new Tid from a raw value.
    ///
    /// # Safety
    ///
    /// `val` must correspond to a valid task descriptor.
    pub unsafe fn from_raw(val: usize) -> Tid {
        Tid(val)
    }

    /// Return the Tid's raw value.
    pub fn raw(self) -> usize {
        self.0
    }
}
