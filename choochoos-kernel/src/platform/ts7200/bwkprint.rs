//! Busy-wait logging primitives.

/// Bust-wait Kernel printing over the TS-7200's COM2.
/// Appends "\n\r" to the output.
#[macro_export]
macro_rules! bwkprintln {
    () => { bwkprintln!("") };
    ($fmt:literal) => { bwkprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        #[cfg(not(feature = "platform-ts7200-emulated"))]
        ts7200::bwprintln!(COM2, $fmt, $($arg)*);
        #[cfg(feature = "platform-ts7200-emulated")]
        ts7200::bwprintln!(COM3, $fmt, $($arg)*);
    }};
}

/// Bust-wait Kernel printing over the TS-7200's COM2.
#[macro_export]
macro_rules! bwkprint {
    () => { bwkprint!("") };
    ($fmt:literal) => { bwkprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        #[cfg(not(feature = "platform-ts7200-emulated"))]
        ts7200::bwprint!(COM2, $fmt, $($arg)*);
        #[cfg(feature = "platform-ts7200-emulated")]
        ts7200::bwprint!(COM3, $fmt, $($arg)*);
    };
}
