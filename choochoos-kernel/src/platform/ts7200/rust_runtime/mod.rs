//! Set up the minimal Rust runtime.
//!
//! e.g: Specifying the panic handler, clearing bss

mod crt0;
mod panic;

/// A static variable where we stash away the LR redboot hands us in _start
static mut REDBOOT_RETURN_ADDRESS: *const core::ffi::c_void = core::ptr::null_mut();
