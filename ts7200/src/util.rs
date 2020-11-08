//! Low-level utilities, useful for bootstrapping.
//!
//! **WARNING:** while these utilities may be useful when bootstrapping a
//! system, they should be used with extreme caution, as they access global
//! hardware resources with no synchronization!

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
macro_rules! bwprintln {
    () => { $crate::bwprintln!("") };
    ($fmt:literal) => { $crate::bwprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::util::BusyWaitLogger
            .write_fmt(format_args!(concat!($fmt, "\n\r"), $($arg)*))
            .unwrap();
    }};
}

/// Debug macro to dump output via COM2 using busy waiting.
#[macro_export]
macro_rules! bwprint {
    () => { $crate::bwprint!("") };
    ($fmt:literal) => { $crate::bwprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        $crate::util::BusyWaitLogger
            .write_fmt(format_args!($fmt, $($arg)*))
            .unwrap();
    }};
}
