//! Types and structures exposed by the `choochoos` kernel.

#![no_std]

/// Function signature which can be spawned by the kernel.
pub type TaskFn = extern "C" fn();

/// Task descriptor handle.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
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

pub mod syscall {
    /// Raw `choochoos` syscall numbers (when calling `swi #x`).
    #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
    #[repr(u8)]
    pub enum SyscallNo {
        Yield       = 0,
        Exit        = 1,
        MyParentTid = 2,
        MyTid       = 3,
        Create      = 4,
        Send        = 5,
        Receive     = 6,
        Reply       = 7,
        AwaitEvent  = 8,
        Perf        = 9,
        Shutdown    = 10,
    }

    impl SyscallNo {
        // Convert
        pub fn from_u8(no: u8) -> Option<SyscallNo> {
            if no > 10 {
                None
            } else {
                // SAFETY: SyscallNo is repr(u8), and was checked to be in bounds
                Some(unsafe { core::mem::transmute(no) })
            }
        }
    }

    pub type Yield = unsafe extern "C" fn();
    pub type Exit = unsafe extern "C" fn();
    pub type MyTid = unsafe extern "C" fn() -> isize;
    pub type MyParentTid = unsafe extern "C" fn() -> isize;
    pub type Create =
        unsafe extern "C" fn(priority: isize, function: Option<extern "C" fn()>) -> isize;
}
