use core::ptr;

use bit_field::BitField;

use choochoos_platform_ts7200::{EventId, Interrupt};

unsafe fn service_timer(base_addr: u32) -> usize {
    use ts7200::constants::timer;

    // any 'ol value will do
    ptr::write_volatile((base_addr + timer::CLR_OFFSET) as *mut u32, 1);
    // no significant volatile data is associated with timer interrupts
    0
}

unsafe fn service_uart(base_addr: u32) -> usize {
    use ts7200::constants::uart;

    let ctlr = (base_addr + uart::CTLR_OFFSET) as *mut u32;
    let intr = (base_addr + uart::INTR_OFFSET) as *mut u32;

    let ret = ptr::read_volatile(intr);

    let u_int_id = ret;
    let mut u_ctlr = ptr::read_volatile(ctlr);

    // TODO: make this more readable

    if u_int_id.get_bit(0) {
        u_ctlr.set_bit(3, false);
    }
    if u_int_id.get_bit(1) {
        u_ctlr.set_bit(4, false);
    }
    if u_int_id.get_bit(2) {
        u_ctlr.set_bit(5, false);
    }
    if u_int_id.get_bit(3) {
        u_ctlr.set_bit(6, false);
    }

    ptr::write_volatile(ctlr, u_ctlr);

    // union UARTIntIDIntClr {
    //     uint32_t raw;
    //     struct {
    //         bool modem : 1;
    //         bool rx : 1;
    //         bool tx : 1;
    //         bool rx_timeout : 1;
    //     } _;
    // };

    // union UARTCtrl {
    //     uint32_t raw;
    //     struct {
    //         bool uart_enable : 1;
    //         bool sir_enable : 1;
    //         bool sir_low_power : 1;
    //         bool enable_int_modem : 1;
    //         bool enable_int_rx : 1;
    //         bool enable_int_tx : 1;
    //         bool enable_int_rx_timeout : 1;
    //         bool loopback_enable : 1;
    //     } _;
    // };

    ret as _
}

unsafe fn service_interrupt(vic_idx: u8) -> (EventId, usize) {
    let interrupt = match Interrupt::from_overall_idx(vic_idx) {
        Some(interrupt) => interrupt,
        None => panic!("unexpected vic_idx {}", vic_idx),
    };

    let volatile_data = match interrupt {
        Interrupt::Tc1Ui => service_timer(ts7200::constants::timer::TIMER1_BASE),
        Interrupt::Tc2Ui => service_timer(ts7200::constants::timer::TIMER2_BASE),
        Interrupt::Tc3Ui => service_timer(ts7200::constants::timer::TIMER3_BASE),
        Interrupt::IntUart1 => service_uart(ts7200::constants::uart::UART1_BASE),
        Interrupt::IntUart2 => service_uart(ts7200::constants::uart::UART2_BASE),
        _ => unimplemented!("unimplemented interrupt source: {:?}", interrupt),
    };

    (EventId::from_interrupt(interrupt), volatile_data)
}

// ---------------------------- public interface ---------------------------- //

pub fn validate_eventid(event_id: usize) -> bool {
    EventId::from_raw(event_id).is_some()
}

pub unsafe fn handle_irq(mut handle_interrupt: impl FnMut(usize, usize)) {
    use ts7200::constants::vic;

    let vic1_bits = ptr::read_volatile((vic::VIC1_BASE + vic::IRQ_STATUS_OFFSET) as *mut u32);
    let vic2_bits = ptr::read_volatile((vic::VIC2_BASE + vic::IRQ_STATUS_OFFSET) as *mut u32);

    for i in 0..32 {
        if vic1_bits & (1 << i) != 0 {
            let (event_id, volatile_data) = service_interrupt(i);
            handle_interrupt(event_id.raw(), volatile_data);
        }
    }
    for i in 0..32 {
        if vic2_bits & (1 << i) != 0 {
            let (event_id, volatile_data) = service_interrupt(32 + i);
            handle_interrupt(event_id.raw(), volatile_data);
        }
    }
}
