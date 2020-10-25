/// Called by the _irq_handler assembly routine
pub unsafe extern "C" fn handle_irq() {
    let kernel = match &mut crate::KERNEL {
        Some(kernel) => kernel,
        None => core::hint::unreachable_unchecked(),
    };

    let _ = kernel;
    // stubbed
}
