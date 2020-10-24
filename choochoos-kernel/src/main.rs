#![no_std]
#![no_main]
#![feature(asm)] // TODO: move all asm to separate .asm file?
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

#[macro_use]
mod kernel_log;

#[cfg(feature = "heap")]
mod heap;

mod kernel;
mod platform;
mod util;

/// There can be only one kernel.
///
/// We use an Option to represent the initial "uninitialized" state, which is
/// later initialized via the `Kernel::init` method.
///
/// This could potentially be replaced with a UnsafeCell<MaybeUninit<Kernel>>,
/// but this approach is fine for now.
static mut KERNEL: Option<kernel::Kernel> = None;

// called from `_start`. See `src/platform/<platform>/rust_runtime/crt0.rs`
fn main() -> isize {
    unsafe { kernel::Kernel::init() }.run();
    0
}
