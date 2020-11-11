//! The choochoos name server implementation.
//!
//! Unlike all other userspace tasks, the [`NameServerTask`] is directly spawned
//! by the Kernel, and is guaranteed to have a
//! [fixed TID](sys::abi::NAMESERVER_TID).

#![deny(missing_docs)]
#![no_std]

use heapless::consts::*;
use heapless::LinearMap;

use sys::abi::NAMESERVER_TID;
use sys::Tid;
use syscall as sys;

const ARENA_SIZE: usize = 1024;
const MAX_REGISTERED_TASKS: usize = 16;
const MAX_NAME_SIZE: usize = ARENA_SIZE / MAX_REGISTERED_TASKS;
#[allow(non_camel_case_types)]
type MAX_REGISTERED_TASKS = U16;

// Messaging protocol:
//
// Request: [ Kind (1 byte) | string ... ]
// - The length of the request is 1 + the length of the string.
//
// Response (RegisterAs): []
// Response (WhoIs):      [] or [ Tid (4 bytes) ]
// - The length of the response encodes whether or not the TID could be found.

#[derive(Debug)]
#[repr(u8)]
enum RequestKind {
    RegisterAs = 0,
    WhoIs      = 1,
}

impl RequestKind {
    fn from_u8(val: u8) -> Option<RequestKind> {
        Some(match val {
            0 => RequestKind::RegisterAs,
            1 => RequestKind::WhoIs,
            _ => return None,
        })
    }
}

struct NameServer {
    arena: [u8; ARENA_SIZE],
    arena_idx: usize,
    registrations: LinearMap<Tid, (usize, usize), MAX_REGISTERED_TASKS>,
}

impl NameServer {
    fn new() -> NameServer {
        NameServer {
            arena: [0; ARENA_SIZE],
            arena_idx: 0,
            registrations: LinearMap::new(),
        }
    }

    fn run(&mut self) -> ! {
        assert_eq!(sys::my_tid(), NAMESERVER_TID);

        let mut msg = [0; MAX_NAME_SIZE];
        'msg: loop {
            let (tid, len) = sys::receive(&mut msg).expect("invalid message");
            let msg = &msg[..len];

            if msg.is_empty() {
                panic!("invalid message")
            }

            let kind = RequestKind::from_u8(msg[0]).expect("invalid message");
            let name = &msg[1..];

            match kind {
                RequestKind::RegisterAs => {
                    let entry_idx = self.arena_idx;
                    let name_len = name.len();
                    self.arena[self.arena_idx..][..name_len].copy_from_slice(&name);
                    self.arena_idx += name_len;
                    self.registrations
                        .insert(tid, (entry_idx, name_len))
                        .expect("no more room in string arena");

                    let _ = sys::reply(tid, &[]);
                }
                RequestKind::WhoIs => {
                    // linear scan through registered names
                    for (&whois_tid, &(entry_idx, len)) in self.registrations.iter() {
                        if &self.arena[entry_idx..][..len] == name {
                            let _ = sys::reply(tid, &whois_tid.raw().to_le_bytes());
                            continue 'msg;
                        }
                    }

                    let _ = sys::reply(tid, &[]);
                }
            }
        }
    }
}

/// Main name server task, implicitly spawned by the kernel at startup.
#[no_mangle]
pub extern "C" fn NameServerTask() -> ! {
    NameServer::new().run()
}

/// Errors which may occur when talking to the Name Server.
#[derive(Debug)]
pub enum Error {
    /// Could not reach the Name Server.
    InvalidNameserver,
}

/// Registers the task id of the caller under the given name.
///
/// On return without error, it is guaranteed that all `who_is()` calls by
/// any task will return the task id of the caller until the
/// registration is overwritten.
///
/// If another task has already registered with the given name, its
/// registration is overwritten.
pub fn register_as(name: impl AsRef<[u8]>) -> Result<(), Error> {
    register_as_impl(name.as_ref())
}

fn register_as_impl(name: &[u8]) -> Result<(), Error> {
    let mut req = [0; MAX_NAME_SIZE];
    req[0] = RequestKind::RegisterAs as _;
    req[1..][..name.len()].copy_from_slice(name);
    let req = &req[0..(name.len() + 1)];

    match sys::send(NAMESERVER_TID, &req, &mut []) {
        Ok(_) => Ok(()),
        Err(sys::error::Send::TidDoesNotExist) => Err(Error::InvalidNameserver),
        Err(_) => panic!("unexpected nameserver error"),
    }
}

/// Asks the name server for the task id of the task that is registered
/// under the given name.
///
/// Whether `who_is()` blocks waiting for a registration or returns with an
/// error, if no task is registered under the given name, is
/// implementation-dependent.
///
/// NOTE: this implementation is non-blocking, and returns None if no task is
/// registered under a given name.
///
/// There is guaranteed to be a unique task id associated with each
/// registered name, but the registered task may change at any time
/// after a call to `who_is()`.
pub fn who_is(name: impl AsRef<[u8]>) -> Result<Option<Tid>, Error> {
    who_is_impl(name.as_ref())
}

fn who_is_impl(name: &[u8]) -> Result<Option<Tid>, Error> {
    let mut req = [0; MAX_NAME_SIZE];
    req[0] = RequestKind::WhoIs as _;
    req[1..][..name.len()].copy_from_slice(name);
    let req = &req[0..(name.len() + 1)];

    const TID_SIZE: usize = core::mem::size_of::<usize>();

    let mut tid = [0; TID_SIZE];
    let tid = match sys::send(NAMESERVER_TID, &req, &mut tid) {
        Ok(len) => match len {
            0 => None,
            TID_SIZE => Some(unsafe { Tid::from_raw(usize::from_le_bytes(tid)) }),
            _ => panic!("unexpected nameserver response"),
        },
        Err(sys::error::Send::TidDoesNotExist) => return Err(Error::InvalidNameserver),
        Err(_) => panic!("unexpected nameserver error"),
    };

    Ok(tid)
}

/// C-FII exposing the interface outlined in the
/// [CS 452 Kernel Description](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html).
pub mod ffi {
    use super::*;

    #[inline(always)]
    unsafe fn strlen(p: *const u8) -> usize {
        let mut n = 0;
        while *p.add(n) != 0 {
            n += 1;
        }
        n
    }

    /// C-FFI wrapper around [`register_as`].
    ///
    /// `name` must be a null terminated C string.
    ///
    /// Returns 0 on success, -1 if the nameserver could not be reached, and
    /// -2 if `name` was null.
    ///
    /// # Safety
    ///
    /// Safe for all values on `name`.
    #[no_mangle]
    pub unsafe extern "C" fn RegisterAs(name: *const u8) -> isize {
        if name.is_null() {
            return -2;
        }

        let name = core::slice::from_raw_parts(name, strlen(name));

        match register_as(name) {
            Ok(()) => 0,
            Err(Error::InvalidNameserver) => -1,
        }
    }

    /// C-FFI wrapper around [`who_is`].
    ///
    /// `name` must be a null terminated C string.
    ///
    /// Returns the Tid on success, -1 if the nameserver could not be reached,
    /// and -2 if `name` was null.
    ///
    /// # Safety
    ///
    /// Safe for all values on `name`.
    #[no_mangle]
    pub unsafe extern "C" fn WhoIs(name: *const u8) -> isize {
        if name.is_null() {
            return -2;
        }

        let name = core::slice::from_raw_parts(name, strlen(name));

        match who_is(name) {
            Ok(Some(tid)) => tid.raw() as isize,
            Ok(None) => -2,
            Err(Error::InvalidNameserver) => -1,
        }
    }
}
