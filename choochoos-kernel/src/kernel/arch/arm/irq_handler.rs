/// Called by the _irq_handler assembly routine
#[no_mangle]
unsafe extern "C" fn handle_interrupt() {
    let kernel = match &mut crate::KERNEL {
        Some(kernel) => kernel,
        None => core::hint::unreachable_unchecked(),
    };

    let _ = kernel;
    // stubbed
}

extern "C" {
    // implemented in asm.s
    pub fn _irq_handler();
}
