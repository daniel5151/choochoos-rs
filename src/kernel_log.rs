//! TODO: use an in-memory buffer for the kernel log instead of immediately
//! dumping via the BusyWaitLogger

// TODO: hook into `log` crate
#[macro_export]
macro_rules! kdebug {
    () => { kdebug!("") };
    ($fmt:literal) => { kdebug!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            use core::fmt::Write;
            ts7200::util::BusyWaitLogger
                .write_fmt(format_args!(
                    // foreground color = yellow
                    concat!("\x1b[33m", "[kdebug][{}:{}][tid={}] ", "\x1b[0m", $fmt, "\n\r"),
                    file!(),
                    line!(),
                    unsafe {
                        $crate::KERNEL
                            .as_ref()
                            .unwrap()
                            .current_tid()
                            .map(|t| t.raw() as isize)
                            .unwrap_or(-1)
                    },
                    $($arg)*
                ))
                .unwrap();
        }
    };
}

/// General Purpose kernel logging mechanism. Appends "\n\r" to the output
#[macro_export]
macro_rules! kprintln {
    () => { kprintln!("") };
    ($fmt:literal) => { kprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        blocking_println!($fmt, $($arg)*)
    }};
}

/// General Purpose kernel logging mechanism.
#[macro_export]
macro_rules! kprint {
    () => { kprint!("") };
    ($fmt:literal) => { kprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        busywait_print!($fmt, $($arg)*)
    };
}
