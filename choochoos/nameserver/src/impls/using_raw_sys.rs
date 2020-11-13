use heapless::consts::*;
use heapless::LinearMap;

use sys::abi::NAMESERVER_TID;
use sys::Tid;
use syscall as sys;

use crate::Error;

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

pub(crate) struct NameServer {
    arena: [u8; ARENA_SIZE],
    arena_idx: usize,
    registrations: LinearMap<Tid, (usize, usize), MAX_REGISTERED_TASKS>,
}

impl NameServer {
    pub(crate) fn new() -> NameServer {
        NameServer {
            arena: [0; ARENA_SIZE],
            arena_idx: 0,
            registrations: LinearMap::new(),
        }
    }

    pub(crate) fn run(&mut self) -> ! {
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
        Err(e) => panic!("unexpected name server error: {:?}", e),
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
            _ => panic!("unexpected name server response"),
        },
        Err(sys::error::Send::TidDoesNotExist) => return Err(Error::InvalidNameserver),
        Err(e) => panic!("unexpected name server error: {:?}", e),
    };

    Ok(tid)
}
