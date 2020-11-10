//! Kernel logging macros.
//!
//! TODO: use an in-memory buffer for the kernel log instead of immediately
//! dumping via the BusyWaitLogger

// TODO: hook into `log` crate
/// Logs messages when the `kdebug` feature is enabled.
#[doc(cfg(feature = "kdebug"))]
#[macro_export]
macro_rules! kdebug {
    () => { kdebug!("") };
    ($fmt:literal) => { kdebug!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        #[cfg(feature = "kdebug")]
        #[allow(unused_unsafe)]
        unsafe {
            use core::fmt::Write;
            crate::platform::bwprint::BusyWaitLogger
                .write_fmt(format_args!(
                    // foreground color = yellow
                    concat!("\x1b[33m", "[kdebug][tid={}][{}:{}] ", "\x1b[0m", $fmt, "\n\r"),
                    match $crate::KERNEL {
                        Some(ref kernel) => {
                            kernel.current_tid()
                                .map(|t| t.raw() as isize)
                                .unwrap_or(-1)
                        }
                        None => unsafe { core::hint::unreachable_unchecked(); },
                    },
                    file!(),
                    line!(),
                    $($arg)*
                ))
                .unwrap();
        }
        #[cfg(not(feature = "kdebug"))]
        {
            let _ = ($fmt, $($arg)*);
        }
    };
}

/// General Purpose kernel logging mechanism. Appends "\n\r" to the output.
// TODO: hook this into an internal log buffer instead of bwprint
#[macro_export]
macro_rules! kprintln {
    () => { kprintln!("") };
    ($fmt:literal) => { kprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        crate::platform::bwprint::bwprintln!($fmt, $($arg)*)
    }};
}

/// General Purpose kernel logging mechanism.
// TODO: hook this into an internal log buffer instead of bwprint
#[macro_export]
macro_rules! kprint {
    () => { kprint!("") };
    ($fmt:literal) => { kprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        crate::platform::bwprint::bwprint!($fmt, $($arg)*)
    };
}
