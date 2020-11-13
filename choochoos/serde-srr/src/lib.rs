//! Send-Receive-Reply Rust types between tasks using [`serde`].
//!
//! At the moment, this crate uses [`postcard`] as the underlying binary
//! de/serialization protocol. In the future, it might make sense to write a
//! custom `serde` data format that is tailored to `choochoos`.

#![no_std]

use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};

use syscall::{self as sys, Tid};

/// An error returned from the [`Sender::send`] method.
#[derive(Debug)]
pub enum SendError {
    /// The underlying `Send` syscall failed.
    Syscall(sys::error::Send),
    /// Postcard de/serialization failed.
    Postcard(postcard::Error),
}

/// An error returned from the [`Receiver::receive`] method.
#[derive(Debug)]
pub enum ReceiveError {
    /// The underlying `Receive` syscall failed.
    Syscall(sys::error::Receive),
    /// Postcard deserialization failed.
    Postcard(postcard::Error),
}

/// An error returned from the [`Receiver::reply`] method.
#[derive(Debug)]
pub enum ReplyError {
    /// The underlying `Reply` syscall failed.
    Syscall(sys::error::Reply),
    /// Postcard serialization failed.
    Postcard(postcard::Error),
}

/// Send `serde`-serializable data types.
pub struct Sender<'a> {
    buf: &'a mut [u8],
}

impl<'a> Sender<'a> {
    /// Construct a new `Sender` using the provided fixed-size buffer.
    ///
    /// The buffer will be used to both serialize message + deserialize
    /// response data, and should be sized appropriately.
    pub fn new(buf: &'a mut [u8]) -> Sender<'a> {
        Sender { buf }
    }

    /// Send a `serde`-serializable data type to the specified task,
    /// blocking until a reply is received.
    #[allow(clippy::needless_lifetimes)] // misfiring?
    pub fn send<'b, Msg, Reply>(&'b mut self, tid: Tid, msg: &Msg) -> Result<Reply, SendError>
    where
        Msg: Serialize,
        Reply: Deserialize<'b>,
    {
        let send_len = {
            to_slice(msg, &mut self.buf)
                .map_err(SendError::Postcard)?
                .len()
        };

        let len = sys::send_shared_buf(tid, &mut self.buf, send_len).map_err(SendError::Syscall)?;

        let res = from_bytes(&self.buf[..len]).map_err(SendError::Postcard)?;

        Ok(res)
    }
}

/// Receive and Reply `serde`-de/serializable data types.
pub struct Receiver<'a> {
    buf: &'a mut [u8],
}

impl<'a> Receiver<'a> {
    /// Construct a new `Sender` using the provided fixed-size buffer.
    ///
    /// The buffer will be used to both deserialize message data +
    /// serialize reply data, and should be sized appropriately.
    pub fn new(buf: &'a mut [u8]) -> Receiver<'a> {
        Receiver { buf }
    }

    /// Receive a `serde`-deserializable data type from another task,
    /// blocking until a message is received.
    pub fn receive<'b, Msg>(&'b mut self) -> Result<(Tid, Msg), ReceiveError>
    where
        Msg: Deserialize<'b>,
    {
        let (tid, len) = sys::receive(&mut self.buf).map_err(ReceiveError::Syscall)?;
        let res = from_bytes(&self.buf[..len]).map_err(ReceiveError::Postcard)?;
        Ok((tid, res))
    }

    /// Send a `serde`-serializable data type to another task, blocking
    /// until the reply is received.
    pub fn reply<Reply>(&mut self, tid: Tid, reply: &Reply) -> Result<(), ReplyError>
    where
        Reply: Serialize,
    {
        let reply = to_slice(reply, self.buf).map_err(ReplyError::Postcard)?;
        sys::reply(tid, reply).map_err(ReplyError::Syscall)?;
        Ok(())
    }
}
