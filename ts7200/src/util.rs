use core::fmt;

use crate::hw::uart::{self, Uart};

/// Implements `fmt::Write` on COM2 using busy-waiting.
pub struct BusyWaitLogger;

impl fmt::Write for BusyWaitLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let uart = unsafe {
            static mut COM2: Option<Uart> = None;
            // lazy initialization
            if COM2.is_none() {
                COM2 = Some(Uart::new(uart::Channel::COM2));
            }

            COM2.as_mut().unwrap()
        };

        for &b in s.as_bytes() {
            if b == b'\n' {
                uart.write_byte_blocking(b'\r');
            }
            uart.write_byte_blocking(b);
        }

        Ok(())
    }
}

/// Debug macro to dump output via COM2 using busy waiting. Appends "\n\r" to
/// the ouptut.
#[macro_export]
macro_rules! blocking_println {
    () => { blocking_println!("") };
    ($fmt:literal) => { blocking_println!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::util::BusyWaitLogger
            .write_fmt(format_args!(concat!($fmt, "\n\r"), $($arg)*))
            .unwrap();
    }};
}

/// Debug macro to dump output via COM2 using busy waiting.
#[macro_export]
macro_rules! blocking_print {
    () => { blocking_print!("") };
    ($fmt:literal) => { blocking_print!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::util::BusyWaitLogger
            .write_fmt(format_args!($fmt, $($arg)*))
            .unwrap();
    }};
}
