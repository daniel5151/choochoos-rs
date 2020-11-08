//! The top-level choochoos userspace crate.
//!
//! This crate compiles down to a `staticlib` which is then linked with the
//! choochoos kernel. It uses `cargo` features to specify which "distro" should
//! be linked in.
//!
//! This crate is mostly just "glue" between a individual distro and shared
//! userspace functionality (e.g: the nameserver/clockserver/uartserver
//! implementations, the userspace panic handler, etc...).

#![feature(doc_cfg)]
#![no_std]

mod panic;

// ensure the nameserver gets linked in
pub use nameserver as _;

#[cfg(feature = "k1")]
#[doc(cfg(feature = "k1"))]
pub mod k1 {
    pub use k1 as _;
}

#[cfg(feature = "k2")]
#[doc(cfg(feature = "k2"))]
pub mod k2 {
    pub use k2 as _;
}
