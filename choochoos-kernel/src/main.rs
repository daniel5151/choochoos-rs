#![no_std]
#![no_main]
#![feature(llvm_asm)] // TODO: move all asm to separate .asm file?
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

// ensure the userspace is linked in
extern crate choochoos_user;

#[macro_use]
extern crate ts7200;

#[macro_use]
mod kernel_log;

mod boilerplate;
mod kernel;

// called from `_start`. See `boilerplate/crt0.rs`
fn main() -> isize {
    unsafe { kernel::Kernel::init() }.run()
}
