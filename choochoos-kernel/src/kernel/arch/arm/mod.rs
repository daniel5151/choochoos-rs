//! Architecture specific code for 32-bit ARM

mod ctx_switch;
mod irq_handler;
mod swi_handler;
mod userstack;

pub mod create_task;

pub use ctx_switch::_activate_task;
pub use userstack::UserStack;

pub unsafe fn init() {
    use ctx_switch::{_irq_handler, _swi_handler};

    // Register exception handlers
    core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);
    core::ptr::write_volatile(0x38 as *mut unsafe extern "C" fn(), _irq_handler);
}
