#![no_std]
#![no_main]
#![feature(const_fn, const_if_match)]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate alloc;

mod pre_init;

pub mod ts7200;
pub mod uart;

fn main() -> isize {
    // Mess around with the UART
    let mut term_uart = unsafe { uart::Uart::new(uart::Channel::COM2) };
    term_uart.set_fifo(false);

    let mut msg = *b"hello World!\n\r";
    msg[0] = b'H';
    term_uart.write_blocking(&msg);

    term_uart.write_blocking(b"> ");
    let c = term_uart.read_byte_blocking();
    // format! uses the allocator
    term_uart.write_blocking(format!("Read byte <{}>\n\r", c).as_bytes());

    0
}
