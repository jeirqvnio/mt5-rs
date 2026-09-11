//! Bars, ticks and the order book.

/// One OHLC bar.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rate {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub tick_volume: i64,
    pub spread: i32,
    pub real_volume: i64,
}

/// A tick, live (`symbol_tick`) or historical (`ticks_*`): the same record.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tick {
    pub time: i64,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume: u64,
    /// Milliseconds — the only sub-second clock MT5 exposes, and the unit the
    /// tick calls take as their `from`.
    pub time_msc: i64,
    pub flags: u32,
    pub volume_real: f64,
}

/// One level of the order book.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BookEntry {
    /// `ENUM_BOOK_TYPE`, eight bytes on the wire.
    pub kind: i64,
    pub price: f64,
    pub volume: i64,
    pub volume_real: f64,
}
