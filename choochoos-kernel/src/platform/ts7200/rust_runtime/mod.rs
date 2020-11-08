//! Set up the minimal Rust runtime.
//!
//! e.g: Specifying the panic handler, clearing bss

/// Life before `main`.
mod crt0;

/// Kernel Panic handler.
// TODO: the core panic handler should probably be generic across architectures
mod panic;

/// A static variable containing the value of the link-register provided by
/// Redboot when `_start` is called.
static mut REDBOOT_RETURN_ADDRESS: *const core::ffi::c_void = core::ptr::null_mut();
