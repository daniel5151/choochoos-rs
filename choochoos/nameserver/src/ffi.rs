//! C-FII exposing the interface outlined in the
//! [CS 452 Kernel Description](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html).

use super::{register_as, who_is, Error};

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
        Ok(Some(tid)) => tid.into() as isize,
        Ok(None) => -2,
        Err(Error::InvalidNameserver) => -1,
    }
}
