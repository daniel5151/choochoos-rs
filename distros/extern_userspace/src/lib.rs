//! A "stub" distro that links against an external userspace library (typically
//! written in C) for it's `FirstUserTask` implementation, while providing all
//! other base OS functionality (such as the `NameServerTask` implementation,
//! syscalls, etc...).

#![no_std]

pub use choochoos as _;
