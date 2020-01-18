#![no_std]
#![no_main]
#![feature(const_fn, const_if_match)]
#![cfg_attr(feature = "heap", feature(alloc_error_handler))]

#[cfg(feature = "heap")]
#[macro_use]
extern crate alloc;

#[macro_use]
mod debug;
mod kernel_log;

#[cfg(feature = "heap")]
mod heap;

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
