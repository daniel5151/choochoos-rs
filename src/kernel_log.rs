//! TODO: use an in-memory buffer for the kernel log instead of immediately
//! dumping via the BusyWaitLogger

// TODO: hook into `log` crate
#[macro_export]
macro_rules! kdebug {
    () => { kdebug!("") };
    ($fmt:literal) => { kdebug!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        crate::busy_wait_log::BusyWaitLogger
            .write_fmt(format_args!(
                // foreground color = blue
                concat!("\x1b[34m", "[kdebug][{}:{}] ", "\x1b[0m", $fmt, "\n\r"),
                file!(),
                line!(),
                $($arg)*
            ))
            .unwrap();
    }};
}

/// General Purpose kernel logging mechanism. Appends "\n\r" to the output
#[macro_export]
macro_rules! kprintln {
    () => { kprintln!("") };
    ($fmt:literal) => { kprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        crate::busy_wait_log::BusyWaitLogger
            .write_fmt(format_args!(concat!($fmt, "\n\r"), $($arg)*))
            .unwrap();
    }};
}

/// General Purpose kernel logging mechanism.
#[macro_export]
macro_rules! kprint {
    () => { kprint!("") };
    ($fmt:literal) => { kprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        use core::fmt::Write;
        crate::busy_wait_log::BusyWaitLogger
            .write_fmt(format_args!($fmt, $($arg)*))
            .unwrap();
    }};
}