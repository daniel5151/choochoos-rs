use heapless::consts::*;
use heapless::LinearMap;
use serde::{Deserialize, Serialize};

use syscall as sys;

use sys::abi::NAMESERVER_TID;
use sys::Tid;

use crate::Error;

const ARENA_SIZE: usize = 1024;
const MAX_REGISTERED_TASKS: usize = 16;
const MAX_MSG_SIZE: usize = ARENA_SIZE / MAX_REGISTERED_TASKS;
#[allow(non_camel_case_types)]
type MAX_REGISTERED_TASKS = U16;

#[derive(Debug, Serialize, Deserialize)]
enum Request<'a> {
    RegisterAs(&'a [u8]),
    WhoIs(&'a [u8]),
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

        let buf = &mut [0; MAX_MSG_SIZE];
        let recv = &mut serde_srr::Receiver::new(buf);

        'msg: loop {
            let (tid, req) = recv.receive::<Request>().expect("invalid message");

            match req {
                Request::RegisterAs(name) => {
                    let entry_idx = self.arena_idx;
                    let name_len = name.len();
                    self.arena[self.arena_idx..][..name_len].copy_from_slice(&name);
                    self.arena_idx += name_len;
                    self.registrations
                        .insert(tid, (entry_idx, name_len))
                        .expect("no more room in string arena");

                    let _ = recv.reply(tid, &());
                }
                Request::WhoIs(name) => {
                    // linear scan through registered names
                    for (&whois_tid, &(entry_idx, len)) in self.registrations.iter() {
                        if &self.arena[entry_idx..][..len] == name {
                            let _ = recv.reply(tid, &Some(whois_tid));
                            continue 'msg;
                        }
                    }

                    let _ = recv.reply(tid, &None::<Tid>);
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
    let buf = &mut [0; MAX_MSG_SIZE];
    let mut sender = serde_srr::Sender::new(buf);
    match sender.send(NAMESERVER_TID, &Request::RegisterAs(name)) {
        Ok(()) => Ok(()),
        Err(serde_srr::SendError::Syscall(sys::error::Send::TidDoesNotExist)) => {
            Err(Error::InvalidNameserver)
        }
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
    let buf = &mut [0; MAX_MSG_SIZE];
    let mut sender = serde_srr::Sender::new(buf);
    let tid = match sender.send(NAMESERVER_TID, &Request::WhoIs(name)) {
        Ok(Some(tid)) => Some(tid),
        Ok(None) => None,
        Err(serde_srr::SendError::Syscall(sys::error::Send::TidDoesNotExist)) => {
            return Err(Error::InvalidNameserver)
        }
        Err(e) => panic!("unexpected name server error: {:?}", e),
    };

    Ok(tid)
}
