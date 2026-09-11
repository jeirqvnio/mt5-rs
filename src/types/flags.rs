//! The two places the protocol really does use a bitmask rather than an
//! enumeration, so neither becomes one.

/// Selects which ticks `ticks_from` and `ticks_range` return.
///
/// A mask, not a choice: `INFO | TRADE` is a legal request. `ALL` is the
/// MQL5 `-1` as it travels, every bit set.
pub mod copy_ticks {
    /// Every tick.
    pub const ALL: u32 = u32::MAX;
    /// Ticks where the bid or the ask moved.
    pub const INFO: u32 = 1;
    /// Ticks where a trade printed.
    pub const TRADE: u32 = 2;
}

/// Bits of [`crate::SymbolInfo::filling_mode`], which says what a contract
/// accepts rather than what an order asks for.
///
/// `RETURN` is never advertised here even where the broker allows it for
/// pending orders, so [`crate::SymbolInfo::supports_filling`] reports it as
/// unsupported and `order_check` is the way to find out.
pub mod filling_mask {
    pub const FOK: i32 = 1;
    pub const IOC: i32 = 2;
    pub const BOC: i32 = 4;
}
