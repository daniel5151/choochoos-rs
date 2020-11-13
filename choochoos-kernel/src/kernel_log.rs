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
        {
            $crate::bwkprintln!(
                "{} {}",
                format_args!(
                    // foreground color = yellow
                    concat!("\x1b[33m", "[kdebug][tid={}][{}:{}]", "\x1b[0m"),
                    unsafe {
                        match $crate::KERNEL {
                            Some(ref kernel) => {
                                kernel.current_tid()
                                    .map(|tid| tid.into() as isize)
                                    .unwrap_or(-1)
                            }
                            None => core::hint::unreachable_unchecked(),
                        }
                    },
                    file!(),
                    line!(),
                ),
                format_args!($fmt, $($arg)*)
            );
        }
        #[cfg(not(feature = "kdebug"))]
        {
            let _ = ($fmt, $($arg)*);
        }
    };
}

/// General Purpose kernel logging mechanism. Appends "\n\r" to the output.
// TODO: hook this up to an internal log buffer instead of directly to bwprint
#[macro_export]
macro_rules! kprintln {
    () => { kprintln!("") };
    ($fmt:literal) => { kprintln!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {{
        $crate::bwkprintln!($fmt, $($arg)*)
    }};
}

/// General Purpose kernel logging mechanism.
// TODO: hook this up to an internal log buffer instead of directly to bwprint
#[macro_export]
macro_rules! kprint {
    () => { kprint!("") };
    ($fmt:literal) => { kprint!($fmt,) };
    ($fmt:literal, $($arg:tt)*) => {
        $crate::bwkprint!($fmt, $($arg)*)
    };
}
