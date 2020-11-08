//! The core choochoos kernel.

#![no_std]
#![no_main]
#![feature(asm, naked_functions)]
#![feature(doc_cfg)]
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

#[macro_use]
mod kernel_log;
mod util;

#[doc(cfg(feature = "heap"))]
#[cfg(feature = "heap")]
mod heap;

/// Platform-independent core Kernel code.
mod kernel;
/// Platform-specific Kernel code (e.g: hardware initialization / handling)
mod platform;

/// There can be only one kernel.
///
/// We use an Option to represent the initial "uninitialized" state, which is
/// later initialized via the `Kernel::init` method.
///
/// This could potentially be replaced with a UnsafeCell<MaybeUninit<Kernel>>,
/// but this approach is fine for now.
static mut KERNEL: Option<kernel::Kernel> = None;

/// Called from `_start`. See `src/platform/<platform>/rust_runtime/crt0.rs`
fn main() -> isize {
    unsafe { kernel::Kernel::init() }.run();
    0
}
