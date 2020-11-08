//! VIC Interrupts.

/// Interrupt sources available on the TS-7200 platform (non-exhaustive).
///
/// Source: EP93xx User's Guide Table 6-1 (section 6.1.2)
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord, Hash)]
pub enum Interrupt {
    /// TC1 under flow interrupt (Timer Counter 1)
    Tc1Ui,
    /// TC2 under flow interrupt (Timer Counter 2)
    Tc2Ui,
    /// UART 1 Receive Interrupt
    Uart1RxIntr1,
    /// UART 1 Transmit Interrupt
    Uart1TxIntr1,
    /// UART 2 Receive Interrupt
    Uart2RxIntr2,
    /// UART 2 Transmit Interrupt
    Uart2TxIntr2,
    /// UART 3 Receive Interrupt
    Uart3RxIntr3,
    /// UART 3 Transmit Interrupt
    Uart3TxIntr3,
    /// TC3 under flow interrupt (Timer Counter 3)
    Tc3Ui,
    /// UART 1 Interrupt
    IntUart1,
    /// UART 2 Interrupt
    IntUart2,
    /// UART 3 Interrupt
    IntUart3,
}

impl Interrupt {
    /// Returns an index from 0..64 corresponding to the interrupt's overall VIC
    /// index (taking daisy-chaining into account).
    pub fn to_overall_idx(self) -> u8 {
        use Interrupt::*;
        match self {
            Tc1Ui => 4,
            Tc2Ui => 5,
            Uart1RxIntr1 => 23,
            Uart1TxIntr1 => 24,
            Uart2RxIntr2 => 25,
            Uart2TxIntr2 => 26,
            Uart3RxIntr3 => 27,
            Uart3TxIntr3 => 28,
            Tc3Ui => 51,
            IntUart1 => 52,
            IntUart2 => 54,
            IntUart3 => 55,
        }
    }

    /// Returns an index from 0..32 corresponding to the interrupt's
    /// VIC-specific index.
    pub fn to_vic_idx(self) -> u8 {
        self.to_overall_idx() % 32
    }

    /// Construct an `Interrupt` from it's overall VIC index.
    pub fn from_overall_idx(idx: u8) -> Option<Interrupt> {
        use Interrupt::*;
        let interrupt = match idx {
            4 => Tc1Ui,
            5 => Tc2Ui,
            23 => Uart1RxIntr1,
            24 => Uart1TxIntr1,
            25 => Uart2RxIntr2,
            26 => Uart2TxIntr2,
            27 => Uart3RxIntr3,
            28 => Uart3TxIntr3,
            51 => Tc3Ui,
            52 => IntUart1,
            54 => IntUart2,
            55 => IntUart3,
            _ => return None,
        };
        Some(interrupt)
    }
}
