//! Low-level assembly routines for entering/exiting the Kernel.

use core::ptr;

use super::userstack::UserStack;

#[inline(never)]
#[naked]
pub unsafe extern "C" fn _activate_task(
    next_sp: ptr::NonNull<UserStack>,
) -> ptr::NonNull<UserStack> {
    let _ = next_sp;
    // `next_sp` is implicitly placed into "r0"

    asm! {
        // save the kernel's context
        "stmfd   sp!,{{r4-r12,lr}}",

        // pop ret addr and spsr from user stack
        // r1 = spsr, r2 = ret addr
        "ldmfd   r0!,{{r1,r2}}",

        // set the spsr to the user's saved spsr
        "msr     spsr,r1",

        // push user ret val to top of kernel stack
        "stmfd   sp!,{{r2}}",

        // Switch to system mode (IRQs disabled)
        "msr     cpsr_c, #0xdf",

        // install user SP
        "mov     sp,r0",

        // restore user registers from stack
        "ldmfd   sp!,{{r0-r12,lr}}",

        // Switch to supervisor mode (IRQs disabled)
        "msr     cpsr_c, #0xd3",

        // jump back to user code, updating the spsr
        "ldmfd   sp!,{{pc}}^",

        // XXX: it would be better to use `in("r0") next_sp` to ensure `next_sp`
        // is placed in `r0`, but at the time of writing, it causes the compiler
        // to segfault...
        //
        // Once this is fixed, it should also be possible to remove the
        // `#[inline(never)]` attribute.

        // in("r0") next_sp
    };

    // the return value is actually provided by the _swi_handler/_irq_handler
    // assembly routines
    unreachable!("cannot re-enter the kernel from _activate_task");
}

#[naked]
pub unsafe extern "C" fn _swi_handler() {
    asm! {
        // Switch to system mode (IRQs disabled)
        // This banks in the user's LR and SP
        "msr     cpsr_c, #0xdf",

        // Stack user registers on user stack
        "stmfd   sp!,{{r0-r12,lr}}",
        "mov     r4,sp", // hold on to user sp

        // Switch to supervisor mode (IRQs disabled)
        // This banks in the kernel's SP and LR.
        "msr     cpsr_c, #0xd3",

        // store user mode spsr and user mode return address
        "mrs     r0,spsr",
        "stmfd   r4!,{{r0, lr}}",

        // r0 = handle_syscall 1st param = syscall number
        "ldr     r0,[lr, #-4]",      // Load the last-executed SWI instr into r0...
        "bic     r0,r0,#0xff000000", // ...and mask off the top 8 bits to get SWI no
        // r1 = handle_syscall 2nd param = user's stack pointer (with saved state)
        "mov     r1,r4",
        "bl      {handle_syscall}",
        // handle_syscall writes the syscall return value directly into the user's
        // stack (i.e: overwriting the value of the saved r0 register)

        // At this point, the user's stack looks like this:
        //
        // +----------- hi mem -----------+
        // | ... rest of user's stack ... |
        // |   [ lr                   ]   |
        // |   [ r0                   ]   |
        // |   ...                        |
        // |   [ r12                  ]   |
        // |   [ ret addr             ]   |
        // |   [ spsr                 ]   |
        // |         <--- sp --->         |
        // | ....... unused stack ....... |
        // +----------- lo mem -----------+

        // Return the final user SP via r0
        "mov     r0, r4",

        // Restore the kernel's context, and return to the caller of _activate_task
        "ldmfd   sp!,{{r4-r12,pc}}",

        handle_syscall = sym super::swi_handler::handle_syscall
    }
}

#[naked]
pub unsafe extern "C" fn _irq_handler() {
    asm! {
        // Switch to system mode (IRQs disabled)
        // This banks in the user's LR and SP
        "msr     cpsr_c, #0xdf",

        // Stack user registers on user stack
        "stmfd   sp!,{{r0-r12,lr}}",
        "mov     r4,sp", // hold on to user sp

        // Switch to irq mode (IRQs disabled)
        // This banks in the irq's SP and LR.
        "msr     cpsr_c, #0xd2",

        // store user mode spsr and return address (compensating with -4)
        "mrs     r0,spsr",
        "sub     lr,lr,#4",
        "stmfd   r4!,{{r0, lr}}",

        // Switch to kernel mode (IRQs disabled)
        // This banks in the irq's SP and LR.
        "msr     cpsr_c, #0xd3",

        // call syscall handler
        "bl      {handle_irq}",

        // Return the final user SP via r0
        "mov     r0, r4",

        // Restore the kernel's context, and return to the caller of _activate_task
        "ldmfd   sp!,{{r4-r12,pc}}",

        handle_irq = sym super::irq_handler::handle_irq
    }
}
