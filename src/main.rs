#![no_std]
#![no_main]
#![feature(const_fn, const_if_match)]
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

// These modules need to come first, as they expose useful macros to the rest of
// the crate
#[macro_use]
mod busy_wait_log;
#[macro_use]
mod kernel_log;

mod boilerplate;

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

    kprintln!("Read byte <{}>", c);

    #[derive(Debug)]
    struct Foo {
        foo: u32,
        bar: u32,
    }

    kdebug!(
        "Zero alloc formatting test: {:#x?}",
        Foo { foo: 123, bar: 456 }
    );

    kprintln!("Press any button to instantly die");
    let _ = term_uart.read_byte_blocking();
    panic!("Critical mission failure!");
}
