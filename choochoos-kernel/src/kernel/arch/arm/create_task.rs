use core::ptr;

use super::userstack::UserStack;

pub unsafe fn fresh_stack(
    start_addr: usize,
    function: unsafe extern "C" fn(),
) -> ptr::NonNull<UserStack> {
    let sp = (start_addr - core::mem::size_of::<UserStack>()) as *mut UserStack;

    let mut stackview = &mut *sp;
    stackview.spsr = 0x50;
    stackview.pc = function;
    for (i, r) in &mut stackview.regs.iter_mut().enumerate() {
        *r = i; // makes debugging a little easier
    }

    stackview.lr = 0xffffffff; // will trigger an error in `ts7200` emulator

    // HACK: used to run old c-based choochoos programs that assumed a
    // statically linked userspace.
    #[cfg(feature = "legacy-implicit-exit")]
    {
        unsafe extern "C" fn _implicit_exit() {
            llvm_asm!("swi #1")
        }
        stackview.lr = _implicit_exit as usize;
    }

    ptr::NonNull::new_unchecked(sp)
}
