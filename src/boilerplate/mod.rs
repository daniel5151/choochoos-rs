//! Sets up all the nitty-gritty boilerplate Rust needs to work.
//!
//! e.g: Specifying the panic handler, clearing bss, (optionally) setting the
//! global allocator

use crate::ffi::Void;

#[cfg(feature = "heap")]
mod heap;

mod crt0;
mod panic;

/// A static variable where we stash away the LR redboot hands us in _start
static mut REDBOOT_RETURN_ADDRESS: *const Void = core::ptr::null_mut();