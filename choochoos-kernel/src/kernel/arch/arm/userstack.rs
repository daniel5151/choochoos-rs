/// Provides a structured view into a suspended user stack.
///
/// Cannot be instantiated directly, and can only be obtained through an unsafe
/// case from a raw task stack pointer.
#[repr(C, align(4))]
#[derive(Debug)]
#[non_exhaustive] // disallow brace initialization
pub struct UserStack {
    pub spsr: usize,
    pub pc: unsafe extern "C" fn(),
    pub regs: [usize; 13],
    pub lr: usize,
    // gets indexed using `get_unchecked`
    pub other_params: [usize; 0],
}

impl UserStack {
    /// Inject a return value into the saved user stack.
    ///
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    // TODO?: support returning structs?
    pub fn inject_return_value<T: Copy>(&mut self, val: T) {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<usize>());
        self.regs[0] = unsafe { *(&val as *const _ as *const usize) };
    }

    /// Extract arguments for a saved user stack.
    pub fn args(&mut self) -> UserStackArgs<'_> {
        UserStackArgs {
            stack: self,
            idx: 0,
        }
    }
}

/// Helper to extract arguments from a user's stack.
pub struct UserStackArgs<'a> {
    stack: &'a UserStack,
    idx: usize,
}

impl<'a> UserStackArgs<'a> {
    /// Obtain a reference to the next argument in the user's stack.
    ///
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    pub unsafe fn extract_ref<T: Copy>(&mut self) -> &'a T {
        assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<usize>());
        let ret = match self.idx {
            0..=3 => &self.stack.regs[self.idx],
            _ => self.stack.other_params.get_unchecked(self.idx - 4),
        };
        self.idx += 1;
        &*(ret as *const usize as *const T)
    }

    /// Obtain a copy of the next argument in the user's stack.
    ///
    /// # Panics
    ///
    /// Currently only supports values with size equal to
    /// `mem::size_of::<usize>()`
    pub unsafe fn extract<T: 'a + Copy>(&mut self) -> T {
        *self.extract_ref()
    }
}
