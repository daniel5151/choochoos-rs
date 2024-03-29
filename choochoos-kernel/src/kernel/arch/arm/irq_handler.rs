/// Called by the [`_irq_handler`](super::ctx_switch::_irq_handler) assembly
/// routine.
pub unsafe extern "C" fn handle_irq() {
    let kernel = match &mut crate::KERNEL {
        Some(kernel) => kernel,
        None => core::hint::unreachable_unchecked(),
    };

    kernel.handle_irq();
}
