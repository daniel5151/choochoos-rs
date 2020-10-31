pub mod timer {
    pub const TIMER1_BASE: u32 = 0x8081_0000;
    pub const TIMER2_BASE: u32 = 0x8081_0020;
    pub const TIMER3_BASE: u32 = 0x8081_0080;

    pub const LDR_OFFSET: u32 = 0x0;
    pub const VAL_OFFSET: u32 = 0x4;
    pub const CTRL_OFFSET: u32 = 0x8;
    pub const CLR_OFFSET: u32 = 0xc;

    pub const ENABLE_MASK: u32 = 0x80;
    pub const MODE_MASK: u32 = 0x40;
    pub const CLKSEL_MASK: u32 = 0x08;
}

pub mod uart {
    pub const UART1_BASE: u32 = 0x808c_0000;
    pub const UART2_BASE: u32 = 0x808d_0000;

    pub const DATA_OFFSET: u32 = 0x0; // low 8 bits
    pub const DATA_MASK: u32 = 0xff;

    pub const RSR_OFFSET: u32 = 0x4; // low 4 bits
    pub const FE_MASK: u8 = 0x1;
    pub const PE_MASK: u8 = 0x2;
    pub const BE_MASK: u8 = 0x4;
    pub const OE_MASK: u8 = 0x8;

    pub const LCRH_OFFSET: u32 = 0x8; // low 7 bits
    pub const BRK_MASK: u8 = 0x1;
    pub const PEN_MASK: u8 = 0x2; // parity enable
    pub const EPS_MASK: u8 = 0x4; // even parity
    pub const STP2_MASK: u8 = 0x8; // 2 stop bits
    pub const FEN_MASK: u8 = 0x10; // fifo
    pub const WLEN_MASK: u8 = 0x60; // word length

    pub const LCRM_OFFSET: u32 = 0xc; // low 8 bits
    pub const BRDH_MASK: u8 = 0xff; // MSB of baud rate divisor

    pub const LCRL_OFFSET: u32 = 0x10; // low 8 bits
    pub const BRDL_MASK: u8 = 0xff; // LSB of baud rate divisor

    pub const CTLR_OFFSET: u32 = 0x14; // low 8 bits
    pub const UARTEN_MASK: u8 = 0x1;
    pub const MSIEN_MASK: u8 = 0x8; // modem status int
    pub const RIEN_MASK: u8 = 0x10; // receive int
    pub const TIEN_MASK: u8 = 0x20; // transmit int
    pub const RTIEN_MASK: u8 = 0x40; // receive timeout int
    pub const LBEN_MASK: u8 = 0x80; // loopback

    pub const FLAG_OFFSET: u32 = 0x18; // low 8 bits
    pub const CTS_MASK: u8 = 0x1;
    pub const DCD_MASK: u8 = 0x2;
    pub const DSR_MASK: u8 = 0x4;
    pub const TXBUSY_MASK: u8 = 0x8;
    pub const RXFE_MASK: u8 = 0x10; // Receive buffer empty
    pub const TXFF_MASK: u8 = 0x20; // Transmit buffer full
    pub const RXFF_MASK: u8 = 0x40; // Receive buffer full
    pub const TXFE_MASK: u8 = 0x80; // Transmit buffer empty

    pub const INTR_OFFSET: u32 = 0x1c;
    pub const INTR_MS: u32 = 0x1;
    pub const INTR_RX: u32 = 0x2;
    pub const INTR_TX: u32 = 0x4;

    pub const DMAR_OFFSET: u32 = 0x28;
}

pub mod vic {
    pub const VIC1_BASE: u32 = 0x800b0000;
    pub const VIC2_BASE: u32 = 0x800c0000;

    /// IRQ Status Register. The VICxIRQStatus register provides the status of
    /// interrupts after IRQ masking.
    ///
    /// Interrupts 0 - 31 are in VIC1IRQStatus.
    /// Interrupts 32 - 63 are in VIC2IRQStatus.
    pub const IRQ_STATUS_OFFSET: u32 = 0x0000;

    /// Interrupt Select Register. The VICxIntSelect register selects whether
    /// the corresponding interrupt source generates an FIQ or an IRQ
    /// interrupt.
    ///
    /// 1 = FIQ interrupt
    /// 0 = IRQ interrupt
    pub const INT_SELECT_OFFSET: u32 = 0x000C;

    /// Interrupt Enable Register. The VICxIntEnable register enables the
    /// interrupt requests by unmasking the interrupt sources.
    ///
    /// On reset, all interrupts are disabled (masked).
    pub const INT_ENABLE_OFFSET: u32 = 0x0010;

    pub const INT_PROTECTION_OFFSET: u32 = 0x20;
}

pub mod syscon {
    pub const DEVICECFG: u32 = 0x80930080;
    pub const SWLOCK: u32 = 0x809300C0;
    pub const HALT: u32 = 0x80930008;
}
