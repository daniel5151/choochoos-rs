#![no_std]
#![no_main]
#![feature(const_fn, const_if_match)]

mod pre_init;

pub mod ts7200;
pub mod uart;

fn main() -> isize {
    // Mess around with the UART
    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };

    let mut msg = *b"Hello World!\n\r";
    msg[0] = term_uart.read_byte_blocking();

    term_uart.set_fifo(false);
    term_uart.write_blocking(&msg);

    0
}
