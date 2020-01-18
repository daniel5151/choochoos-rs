//! Sets up all the nitty-gritty boilerplate Rust needs to work.
//!
//! e.g: Specifying the panic handler, clearing bss, (optionally) setting the
//! global allocator

#[cfg(feature = "heap")]
mod heap;

mod crt0;
mod panic;
