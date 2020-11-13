//! The choochoos name server implementation.
//!
//! Unlike all other userspace tasks, the [`NameServerTask`] is directly spawned
//! by the Kernel, and is guaranteed to have a
//! [fixed TID](syscall::abi::NAMESERVER_TID).
//!
//! This crate contains two name server implementations: one which implements a
//! bespoke messaging protocol, and one which uses the `serde-ssr` crate to send
//! `serde` compatible Rust types between tasks using a multi-purpose binary
//! serialization protocol.
//!
//! By default, the raw SSR syscall based implementation is used. To enable the
//! `serde-ssr` implementation, disable default features and enable the
//! `using-serde` feature.

#![deny(missing_docs)]
#![feature(doc_cfg)]
#![no_std]

pub mod ffi;
mod impls;

cfg_if::cfg_if! {
    if #[cfg(feature = "using-raw-sys")] {
        use impls::using_raw_sys as ns;
    } else if #[cfg(feature = "using-serde")] {
        use impls::using_serde as ns;
    } else {
        compile_error!("must specify a `using-` feature")
    }
}

pub use ns::{register_as, who_is};

/// Errors which may occur when talking to the Name Server.
#[derive(Debug)]
pub enum Error {
    /// Could not reach the Name Server.
    InvalidNameserver,
}

/// Main name server task, implicitly spawned by the kernel at startup.
#[no_mangle]
pub extern "C" fn NameServerTask() -> ! {
    ns::NameServer::new().run()
}
