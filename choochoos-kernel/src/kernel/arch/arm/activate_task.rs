use core::ptr;

use super::userstack::UserStack;

extern "C" {
    // implemented in asm.s
    pub fn _activate_task(sp: ptr::NonNull<UserStack>) -> ptr::NonNull<UserStack>;
}
