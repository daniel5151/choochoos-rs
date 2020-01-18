//! Low-level abstractions over the TS-7200's hardware.

#![no_std]
#![feature(const_fn, const_if_match)] // TODO: revisit if this is really needed

pub mod constants;
pub mod hw;
pub mod util;
