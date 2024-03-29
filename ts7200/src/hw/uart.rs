use core::ptr;

use crate::constants::uart;

/// Available COM channels on the TS7200 board
pub enum Channel {
    COM1,
    COM2,
    #[doc(hidden)]
    COM3,
}

/// Provides a low-level interface to the UART devices on the TS-7200 board.
// TODO: track order of LCH(L/M/H) register accesses to enforce proper access
// patterns
pub struct Uart {
    base: u32,
    // This type has interior mutability in the form of the UART's hardware.
    _not_sync: core::marker::PhantomData<core::cell::UnsafeCell<()>>,
}

impl Uart {
    /// Creates a new UART wrapping the specified channel.
    ///
    /// # Safety
    ///
    /// There should only be a single Uart struct acting on a physical UART at
    /// any given time. This type does not have any internal synchronization,
    /// and may result in "spooky action at a distance" if multiple instances
    /// are used at the same time!
    pub const unsafe fn new(channel: Channel) -> Uart {
        Uart {
            base: match channel {
                Channel::COM1 => uart::UART1_BASE,
                Channel::COM2 => uart::UART2_BASE,
                Channel::COM3 => uart::UART2_BASE + 0x10000,
            },
            _not_sync: core::marker::PhantomData,
        }
    }

    /// Sets the FIFO enable bit
    pub fn set_fifo(&mut self, state: bool) {
        let line = (self.base + uart::LCRH_OFFSET) as *mut u8;

        unsafe {
            let mut buf = ptr::read_volatile(line);
            buf = if state {
                buf | uart::FEN_MASK
            } else {
                buf & !uart::FEN_MASK
            };
            ptr::write_volatile(line as _, buf);
        }
    }

    /// Reads a byte by busy-waiting until the UART receives data.
    pub fn read_byte_blocking(&self) -> u8 {
        let flags = (self.base + uart::FLAG_OFFSET) as *const u8;
        let data = (self.base + uart::DATA_OFFSET) as *mut u8;

        unsafe {
            while ptr::read_volatile(flags) & uart::RXFF_MASK == 0 {}
            ptr::read_volatile(data) as u8
        }
    }

    /// Writes a byte by busy-waiting until the UART is ready to accept data.
    pub fn write_byte_blocking(&self, b: u8) {
        let flags = (self.base + uart::FLAG_OFFSET) as *const u8;
        let data = (self.base + uart::DATA_OFFSET) as *mut u8;

        unsafe {
            while ptr::read_volatile(flags) & uart::TXFF_MASK != 0 {}
            ptr::write_volatile(data, b)
        };
    }

    /// Writes a string by busy-waiting until the UART has outputted the
    /// entirety string.
    pub fn write_blocking(&self, buf: &[u8]) {
        for b in buf {
            self.write_byte_blocking(*b);
        }
    }
}
