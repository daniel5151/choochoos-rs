//! A "stub" distro that links against an external userspace library (typically
//! written in C) for it's `FirstUserTask` implementation, while providing all
//! other base OS functionality (such as the `NameServerTask` implementation,
//! syscalls, etc...).
//!
//! Passing `EXTERN_DISTRO=foo` will link with `./bin/libfoo.a`, which should be
//! copied into the `./bin/` directory prior to building this crate.

#![no_std]

pub use choochoos as _;
