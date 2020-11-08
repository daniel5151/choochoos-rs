//! The choochoos name server.
//!
//! Unlike all other userspace tasks, the [`NameServerTask`] is directly spawned
//! by the Kernel, and is guaranteed to have a
//! [fixed TID](sys::abi::NAMESERVER_TID).

#![no_std]

use owo_colors::OwoColorize;

use sys::Tid;
use ts7200::bwprintln;

/// WARNING: Currently unimplemented.
#[no_mangle]
pub extern "C" fn NameServerTask() {
    bwprintln!(
        "{}",
        "WARNING: Name Server is currently not implemented!".yellow()
    );
    sys::exit();
}

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
