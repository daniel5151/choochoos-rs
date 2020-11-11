//! Low-level utilities, useful for bootstrapping.
//!
//! **WARNING:** while these utilities may be useful when bootstrapping a
//! system, they should be used with extreme caution, as they access global
//! hardware resources with no synchronization!

use core::fmt;

use crate::hw::uart::{self, Uart};

/// Wrapper around a raw UART that implements `fmt::Write` using busy-waiting.
pub struct BusyWaitLogger {
    uart: Uart,
}

impl BusyWaitLogger {
    /// Create a new BusyWaitLogger.
    ///
    /// # Safety
    ///
    /// There should only be a single Uart struct acting on a physical UART at
    /// any given time. This type does not have any internal synchronization,
    /// and may result in "spooky action at a distance" if multiple instances
    /// are used at the same time!
    pub unsafe fn new(channel: uart::Channel) -> BusyWaitLogger {
        BusyWaitLogger {
            uart: Uart::new(channel),
        }
    }
}

impl fmt::Write for BusyWaitLogger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            if b == b'\n' {
                self.uart.write_byte_blocking(b'\r');
            }
            self.uart.write_byte_blocking(b);
        }

        Ok(())
    }
}

/// Debug macro to output data over a COM channel using busy waiting.
/// Appends "\n\r" to the ouptut.
///
/// e.g:
///
/// ```rust
/// bwprintln!(COM2, "Hello {}!", "World");
/// ```
#[macro_export]
macro_rules! bwprintln {
    (@no_com) => { compile_error!("Missing COM argument!"); };

    () => { $crate::bwprintln!(@no_com) };
    ($fmt:literal) => { $crate::bwprintln!(@no_com) };

    ($com:ident) => { $crate::bwprintln!($com, "") };
    ($com:ident, $fmt:literal) => { $crate::bwprintln!($com, $fmt,) };
    ($com:ident, $fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::hw::uart;
        unsafe { $crate::util::BusyWaitLogger::new(uart::Channel::$com) }
            .write_fmt(format_args!(concat!($fmt, "\n\r"), $($arg)*))
            .unwrap();
    }};
}

/// Debug macro to output data over a COM channel using busy waiting.
///
/// e.g:
///
/// ```rust
/// bwprint!(COM2, "Hello {}!", "World");
/// ```
#[macro_export]
macro_rules! bwprint {
    (@no_com) => { compile_error!("Missing COM argument!"); };

    () => { $crate::bwprint!(@no_com) };
    ($fmt:literal) => { $crate::bwprint!(@no_com) };

    ($com:ident) => { $crate::bwprint!($com, "") };
    ($com:ident, $fmt:literal) => { $crate::bwprint!($com, $fmt,) };
    ($com:ident, $fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::hw::uart;
        unsafe { $crate::util::BusyWaitLogger::new(uart::Channel::$com) }
            .write_fmt(format_args!($fmt, $($arg)*))
            .unwrap();
    }};
}
