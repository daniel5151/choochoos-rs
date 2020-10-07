#![no_std]

mod panic;

#[cfg(feature = "k1")]
extern crate k1;

#[cfg(feature = "k2")]
extern crate k2;
