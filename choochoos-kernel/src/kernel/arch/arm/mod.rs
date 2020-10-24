//! Architecture specific code for 32-bit ARM

mod activate_task;
mod irq_handler;
mod swi_handler;
mod userstack;

pub mod create_task;

pub use activate_task::_activate_task;
pub use userstack::{UserStack, UserStackArgs};

pub unsafe fn init() {
    use swi_handler::_swi_handler;

    // Register interrupt handlers
    core::ptr::write_volatile(0x28 as *mut unsafe extern "C" fn(), _swi_handler);
}
