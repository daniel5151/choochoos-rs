//! The choochoos name server implementation.
//!
//! Unlike all other userspace tasks, the [`NameServerTask`] is directly spawned
//! by the Kernel, and is guaranteed to have a
//! [fixed TID](sys::abi::NAMESERVER_TID).
//!
//! See the [`nameserver_api`] crate for the name server's public-facing API.

#![no_std]

use owo_colors::OwoColorize;

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
