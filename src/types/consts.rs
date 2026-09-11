//! MQL5 enumerations, as the integers the wire carries.

/// `COPY_TICKS_*` selector for the tick calls.
pub mod copy_ticks {
    pub const ALL: u32 = u32::MAX; // -1
    pub const INFO: u32 = 1;
    pub const TRADE: u32 = 2;
}

/// `ENUM_TRADE_REQUEST_ACTIONS`: what a [`crate::TradeRequest`] asks for.
pub mod trade_action {
    pub const DEAL: u32 = 1;
    pub const PENDING: u32 = 5;
    pub const SLTP: u32 = 6;
    pub const MODIFY: u32 = 7;
    pub const REMOVE: u32 = 8;
    pub const CLOSE_BY: u32 = 10;
}

/// `ENUM_ORDER_TYPE`: market buy and sell, and the six pending kinds.
pub mod order_type {
    pub const BUY: u32 = 0;
    pub const SELL: u32 = 1;
    pub const BUY_LIMIT: u32 = 2;
    pub const SELL_LIMIT: u32 = 3;
    pub const BUY_STOP: u32 = 4;
    pub const SELL_STOP: u32 = 5;
    pub const BUY_STOP_LIMIT: u32 = 6;
    pub const SELL_STOP_LIMIT: u32 = 7;
    pub const CLOSE_BY: u32 = 8;
}

/// `ENUM_ORDER_TYPE_FILLING`: what to do when the book cannot fill the
/// whole volume. See [`crate::SymbolInfo::preferred_filling`].
pub mod order_filling {
    pub const FOK: u32 = 0;
    pub const IOC: u32 = 1;
    pub const RETURN: u32 = 2;
    pub const BOC: u32 = 3;
}

/// `ENUM_ORDER_TYPE_TIME`: how long a pending order lives.
pub mod order_time {
    pub const GTC: u32 = 0;
    pub const DAY: u32 = 1;
    pub const SPECIFIED: u32 = 2;
    pub const SPECIFIED_DAY: u32 = 3;
}

/// `ENUM_POSITION_TYPE`: which way a position faces.
pub mod position_type {
    pub const BUY: i32 = 0;
    pub const SELL: i32 = 1;
}

/// `ENUM_DEAL_TYPE`: what a deal was — a trade, or a balance operation such
/// as a credit, a charge or a commission.
pub mod deal_type {
    pub const BUY: i32 = 0;
    pub const SELL: i32 = 1;
    pub const BALANCE: i32 = 2;
    pub const CREDIT: i32 = 3;
    pub const CHARGE: i32 = 4;
    pub const CORRECTION: i32 = 5;
    pub const BONUS: i32 = 6;
    pub const COMMISSION: i32 = 7;
}

/// `ENUM_DEAL_ENTRY`: whether a deal opened a position, closed one, or
/// reversed it.
pub mod deal_entry {
    pub const IN: i32 = 0;
    pub const OUT: i32 = 1;
    pub const INOUT: i32 = 2;
    pub const OUT_BY: i32 = 3;
}

/// Bits of `SymbolInfo::filling_mode`.
pub mod filling_mask {
    pub const FOK: i32 = 1;
    pub const IOC: i32 = 2;
    pub const BOC: i32 = 4;
}
