//! The choochoos userspace library.
//!
//! Provides common choochoos functionality shared across all userspace distros.
//! e.g: syscall implementations, panic handler, nameserver implementation,
//! etc...

#![deny(missing_docs)]
#![no_std]

pub use nameserver as _;

mod panic;

/// (re-export of [`nameserver`])
/// The choochoos nameserver API.
pub mod ns {
    pub use nameserver::{register_as, who_is};
}

/// (re-export of [`syscall`])
/// Safe wrappers around choochoos syscalls.
pub mod sys {
    pub use syscall::*;
}

/// (re-export of [`serde_srr`])
/// Send-Receive-Reply Rust types between tasks using [`serde`].
pub mod serde_srr {
    pub use syscall::*;
}
