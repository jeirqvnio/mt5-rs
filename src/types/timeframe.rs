//! MQL5 `ENUM_TIMEFRAMES`.

/// A chart period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M10,
    M12,
    M15,
    M20,
    M30,
    H1,
    H2,
    H3,
    H4,
    H6,
    H8,
    H12,
    D1,
    W1,
    MN1,
}

impl Timeframe {
    /// The integer the terminal expects on the wire.
    pub fn code(self) -> u32 {
        match self {
            Timeframe::M1 => 1,
            Timeframe::M2 => 2,
            Timeframe::M3 => 3,
            Timeframe::M4 => 4,
            Timeframe::M5 => 5,
            Timeframe::M6 => 6,
            Timeframe::M10 => 10,
            Timeframe::M12 => 12,
            Timeframe::M15 => 15,
            Timeframe::M20 => 20,
            Timeframe::M30 => 30,
            Timeframe::H1 => 16385,
            Timeframe::H2 => 16386,
            Timeframe::H3 => 16387,
            Timeframe::H4 => 16388,
            Timeframe::H6 => 16390,
            Timeframe::H8 => 16392,
            Timeframe::H12 => 16396,
            Timeframe::D1 => 16408,
            Timeframe::W1 => 32769,
            Timeframe::MN1 => 49153,
        }
    }

    /// Bar length in seconds; `MN1` is not constant.
    pub fn seconds(self) -> Option<i64> {
        Some(match self {
            Timeframe::M1 => 60,
            Timeframe::M2 => 120,
            Timeframe::M3 => 180,
            Timeframe::M4 => 240,
            Timeframe::M5 => 300,
            Timeframe::M6 => 360,
            Timeframe::M10 => 600,
            Timeframe::M12 => 720,
            Timeframe::M15 => 900,
            Timeframe::M20 => 1200,
            Timeframe::M30 => 1800,
            Timeframe::H1 => 3600,
            Timeframe::H2 => 7200,
            Timeframe::H3 => 10800,
            Timeframe::H4 => 14400,
            Timeframe::H6 => 21600,
            Timeframe::H8 => 28800,
            Timeframe::H12 => 43200,
            Timeframe::D1 => 86400,
            Timeframe::W1 => 604800,
            Timeframe::MN1 => return None,
        })
    }
}
