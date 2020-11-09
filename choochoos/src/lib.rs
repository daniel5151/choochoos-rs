//! The choochoos userspace support library.
//!
//! Provides common choochoos functionality shared across distros. e.g: syscall
//! implementations, panic handler, nameserver implementation, etc...

#![deny(missing_docs)]
#![no_std]

pub use nameserver as _;

mod panic;

/// The choochoos nameserver API.
/// (re-exported from the [`nameserver`] crate)
pub mod ns {
    pub use nameserver::{register_as, who_is};
}

/// Safe wrappers around choochoos syscalls.
/// (re-exported from the [`syscall`] crate)
pub mod sys {
    pub use syscall::*;
}
