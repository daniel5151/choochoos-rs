//! Slices pointing into userspace.
//!
//! These types do _not_ implement deref to `&[T]` and `&mut [T]`, as this could
//! lead to two Rust slices which alias the same memory. For example, consider a
//! `Send` syscall that uses the same buffer for it's `msg` and `reply`. Naively
//! dereferencing the corresponding `UserSlice`s to Rust slices and then using
//! `copy_from_slice` would result in undefined behavior, as `copy_from_slice`
//! is semantically equivalent to `memcpy`, which does not allow the source and
//! destination pointers to overlap!
#![allow(dead_code)]

// NOTE: at some point, this module might require `arch` or `platform` specific
// logic. At that point, it should be moved out of `util`, and into an
// appropriate folder.

use core::fmt::{self, Debug};
use core::ptr;

/// A reference to a slice in userspace.
#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct UserSlice<T> {
    data: ptr::NonNull<T>,
    len: usize,
}

impl<T: Debug> Debug for UserSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_raw_slice(f, self.data, self.len)
    }
}

impl<T> UserSlice<T> {
    pub fn empty() -> UserSlice<T> {
        UserSlice {
            data: ptr::NonNull::dangling(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }
}

/// A reference to a mutable slice in userspace.
#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct UserSliceMut<T> {
    data: ptr::NonNull<T>,
    len: usize,
}

impl<T: Debug> Debug for UserSliceMut<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_raw_slice(f, self.data, self.len)
    }
}

impl<T> UserSliceMut<T> {
    pub fn empty() -> UserSliceMut<T> {
        UserSliceMut {
            data: ptr::NonNull::dangling(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *mut T {
        self.data.as_ptr()
    }

    /// Unlike `core::slice::copy_from_slice`, this method will _not_ panic if
    /// the slices are different sized, and will instead copy most bytes it can
    /// from the larger buffer into the smaller one. Returns the number of
    /// elements that were copied.
    pub fn copy_from_slice_min(&mut self, src: UserSlice<T>) -> usize {
        let len = src.len.min(self.len);
        unsafe {
            ptr::copy(src.data.as_ptr(), self.data.as_ptr(), len);
        }
        len
    }
}

/// Forms a `UserSlice` from a pointer and length.
pub unsafe fn from_raw_parts<T>(data: ptr::NonNull<T>, len: usize) -> UserSlice<T> {
    UserSlice { data, len }
}

/// Forms a `UserSliceMut<T>` from a pointer and length.
pub unsafe fn from_raw_parts_mut<T>(data: ptr::NonNull<T>, len: usize) -> UserSliceMut<T> {
    UserSliceMut { data, len }
}

fn fmt_raw_slice<T: Debug>(
    f: &mut fmt::Formatter<'_>,
    ptr: ptr::NonNull<T>,
    len: usize,
) -> fmt::Result {
    // special code-path for types that might be ascii strings
    // TODO: use Bstr instead, since it has nicer formatting.
    if core::mem::size_of::<T>() == 1 {
        use bstr::ByteSlice;

        let slice = unsafe { core::slice::from_raw_parts(ptr.as_ptr() as *const u8, len) };
        Debug::fmt(slice.as_bstr(), f)
    } else {
        let slice = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), len) };
        Debug::fmt(slice, f)
    }
}
