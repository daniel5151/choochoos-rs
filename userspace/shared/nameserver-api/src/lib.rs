//! The choochoos name server API.
//!
//! This crate implements the stable userspace interface for the name server.
//!
//! See the `nameserver` crate for the name server task's actual implementation.

#![no_std]

use sys::Tid;

/// Errors which may occur when talking to the Name Server.
pub enum Error {
    /// Could not reach the Name Server.
    InvalidNameserver,
}

/// Registers the task id of the caller under the given name.
///
/// On return without error, it is guaranteed that all `who_is()` calls by
/// any task will return the task id of the caller until the
/// registration is overwritten.
///
/// If another task has already registered with the given name, its
/// registration is overwritten.
pub fn register_as(name: &str) -> Result<(), Error> {
    unimplemented!("called register_as(name: {:?})", name)
}

/// Asks the name server for the task id of the task that is registered
/// under the given name.
///
/// Whether `who_is()` blocks waiting for a registration or returns with an
/// error, if no task is registered under the given name, is
/// implementation-dependent.
///
/// There is guaranteed to be a unique task id associated with each
/// registered name, but the registered task may change at any time
/// after a call to `who_is()`.
///
/// _NOTE:_ `who_is()` is actually a wrapper around a raw `send()` to the
/// name server.
pub fn who_is(name: &str) -> Result<Tid, Error> {
    unimplemented!("called who_is(name: {:?})", name)
}
