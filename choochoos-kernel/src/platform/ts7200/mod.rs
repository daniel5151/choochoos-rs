//! Platform specific support for the
//! [TS-7200 Single Board Computer](https://www.embeddedarm.com/products/TS-7200).

mod rust_runtime;

pub mod bwkprint;
pub mod interrupts;

use core::ptr;
use core::time::Duration;

pub unsafe fn initialize() {
    {
        use ts7200::constants::syscon::*;
        // unlock system controller sw lock
        ptr::write_volatile(SWLOCK as *mut u32, 0xaa);
        // enable halt/standby magic addresses
        let device_cfg = ptr::read_volatile(DEVICECFG as *mut u32);
        ptr::write_volatile(DEVICECFG as *mut u32, device_cfg | 1);
        // system controller re-locks itself
    }

    {
        use ts7200::constants::vic::*;
        use ts7200::interrupts::Interrupt;

        // enable protection (prevents user tasks from poking VIC registers)
        ptr::write_volatile((VIC1_BASE + INT_PROTECTION_OFFSET) as *mut u32, 1);
        ptr::write_volatile((VIC2_BASE + INT_PROTECTION_OFFSET) as *mut u32, 1);

        // all interrupts are handled as IRQs
        ptr::write_volatile((VIC1_BASE + INT_SELECT_OFFSET) as *mut u32, 0);
        ptr::write_volatile((VIC2_BASE + INT_SELECT_OFFSET) as *mut u32, 0);
        // enable timer2 interrupts
        ptr::write_volatile(
            (VIC1_BASE + INT_ENABLE_OFFSET) as *mut u32,
            1 << Interrupt::Tc2Ui.to_vic_idx(),
        );
        // enable uart1 and uart2 combined interrupt
        ptr::write_volatile(
            (VIC2_BASE + INT_ENABLE_OFFSET) as *mut u32,
            (1 << Interrupt::IntUart1.to_vic_idx()) | (1 << Interrupt::IntUart2.to_vic_idx()),
        );
    }

    {
        use ts7200::constants::timer::*;

        // initialize timer 3 to count down from UINT32_MAX at 508KHz
        ptr::write_volatile((TIMER3_BASE + CTRL_OFFSET) as *mut u32, 0);
        ptr::write_volatile((TIMER3_BASE + LDR_OFFSET) as *mut u32, core::u32::MAX);
        ptr::write_volatile(
            (TIMER3_BASE + CTRL_OFFSET) as *mut u32,
            ENABLE_MASK | CLKSEL_MASK,
        );
    }
}

pub unsafe fn teardown() {
    use owo_colors::OwoColorize;
    ts7200::bwprintln!(
        COM2,
        "{}",
        "WARNING: ts7200 teardown isn't implemented, real hardware may require a hard reset!"
            .yellow()
    );
}

pub unsafe fn idle_task() -> Duration {
    // This is pretty neat.
    //
    // We request the system controller to put us into a halt state,
    // and to wake up up when an IRQ happens. All good right? But
    // hey, we're in the kernel, and aren't currently accepting
    // IRQs, so this shouldn't work, right?
    //
    // Wrong!
    //
    // The system controller will freeze the PC at this line, and
    // once an IRQ fires, it simply resumes the PC, _without_
    // jumping to the IRQ handler! Instead, we manually invoke the
    // kernel's interrupt handler, which will unblock any blocked
    // tasks.

    use ts7200::constants::syscon;
    ptr::read_volatile(syscon::HALT as *mut u32);

    // XXX: actually track time asleep
    let time_asleep = Duration::new(0, 0);

    let kernel = match &mut crate::KERNEL {
        Some(kernel) => kernel,
        None => core::hint::unreachable_unchecked(),
    };

    kernel.handle_irq();

    time_asleep
}
