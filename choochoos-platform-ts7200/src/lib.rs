//! `choochoos` platform support for the TS-7200.

#![no_std]
#![deny(missing_docs)]

pub use ts7200::interrupts::Interrupt;

/// TS-7200 specific EventIds
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub struct EventId {
    /// The TS7200 only has 64 VIC interrupt sources, and fortunately, source 0
    /// is unused. As such, we can keep things simple and map Interrupt indexes
    /// directly to a `NonZeroU8`.
    ///
    /// Using `NonZeroU8` enables `mem::size_of::<Option<EventId>>() == 1`.
    inner: core::num::NonZeroU8,
}

impl EventId {
    /// Construct an `EventId` corresponding to a particular Interrupt.
    pub fn from_interrupt(interrupt: Interrupt) -> EventId {
        EventId {
            inner: core::num::NonZeroU8::new(interrupt.to_overall_idx()).unwrap(),
        }
    }

    /// Retrieve a raw `usize` value which can be used in the `AwaitEvent`
    /// syscall.
    pub fn raw(self) -> usize {
        self.inner.get() as usize
    }

    /// Construct an `EventId` from a raw `usize`.
    pub fn from_raw(event_id: usize) -> Option<EventId> {
        use core::convert::TryInto;

        let interrupt = Interrupt::from_overall_idx(event_id.try_into().ok()?)?;
        Some(EventId::from_interrupt(interrupt))
    }
}
