#![no_std]

mod panic;

// ensure the nameserver gets linked in
pub use nameserver as _;

#[cfg(feature = "k1")]
extern crate k1;

#[cfg(feature = "k2")]
extern crate k2;
