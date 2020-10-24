//! Slices which point into userspace.
//!
//! `UserSlice<T>` and `UserSliceMut<T>` are basically identical to `&[T]` and
//! `&mut [T]`, except they are `Copy` and `Clone` as well.

// NOTE: at some point, this module might require `arch` or `platform` specific
// logic. At that point, it should be moved out of `util`, and into an
// appropriate folder.

use core::ptr;

/// A `&[T]` which points into userspace.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct UserSlice<T> {
    data: ptr::NonNull<T>,
    len: usize,
}

impl<T> UserSlice<T> {
    pub fn empty() -> UserSlice<T> {
        UserSlice {
            data: ptr::NonNull::dangling(),
            len: 0,
        }
    }
}

impl<T> core::ops::Deref for UserSlice<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T> AsRef<[T]> for UserSlice<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr(), self.len) }
    }
}

/// A `&mut [T]` which points into userspace.
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct UserSliceMut<T> {
    data: ptr::NonNull<T>,
    len: usize,
}

impl<T> UserSliceMut<T> {
    pub fn empty() -> UserSliceMut<T> {
        UserSliceMut {
            data: ptr::NonNull::dangling(),
            len: 0,
        }
    }
}

impl<T> core::ops::Deref for UserSliceMut<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_ref()
    }
}

impl<T> core::ops::DerefMut for UserSliceMut<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T> AsRef<[T]> for UserSliceMut<T> {
    fn as_ref(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_ptr(), self.len) }
    }
}

impl<T> AsMut<[T]> for UserSliceMut<T> {
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_ptr(), self.len) }
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
