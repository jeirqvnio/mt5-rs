//! `TRADE_RETCODE_*` from the trade server.

pub const REQUOTE: u32 = 10004;
pub const REJECT: u32 = 10006;
pub const CANCEL: u32 = 10007;
pub const PLACED: u32 = 10008;
pub const DONE: u32 = 10009;
pub const DONE_PARTIAL: u32 = 10010;
pub const ERROR: u32 = 10011;
pub const TIMEOUT: u32 = 10012;
pub const INVALID: u32 = 10013;
pub const INVALID_VOLUME: u32 = 10014;
pub const INVALID_PRICE: u32 = 10015;
pub const INVALID_STOPS: u32 = 10016;
pub const TRADE_DISABLED: u32 = 10017;
pub const MARKET_CLOSED: u32 = 10018;
pub const NO_MONEY: u32 = 10019;
pub const PRICE_CHANGED: u32 = 10020;
pub const PRICE_OFF: u32 = 10021;
pub const INVALID_EXPIRATION: u32 = 10022;
pub const ORDER_CHANGED: u32 = 10023;
pub const TOO_MANY_REQUESTS: u32 = 10024;
pub const NO_CHANGES: u32 = 10025;
pub const SERVER_DISABLES_AT: u32 = 10026;
pub const CLIENT_DISABLES_AT: u32 = 10027;
pub const LOCKED: u32 = 10028;
pub const FROZEN: u32 = 10029;
pub const INVALID_FILL: u32 = 10030;
pub const CONNECTION: u32 = 10031;
pub const ONLY_REAL: u32 = 10032;
pub const LIMIT_ORDERS: u32 = 10033;
pub const LIMIT_VOLUME: u32 = 10034;
pub const INVALID_ORDER: u32 = 10035;
pub const POSITION_CLOSED: u32 = 10036;
pub const INVALID_CLOSE_VOLUME: u32 = 10038;
pub const CLOSE_ORDER_EXIST: u32 = 10039;
pub const LIMIT_POSITIONS: u32 = 10040;
pub const REJECT_CANCEL: u32 = 10041;
pub const LONG_ONLY: u32 = 10042;
pub const SHORT_ONLY: u32 = 10043;
pub const CLOSE_ONLY: u32 = 10044;
pub const FIFO_CLOSE: u32 = 10045;
pub const HEDGE_PROHIBITED: u32 = 10046;

/// Whether the server took the order (placed, done, or partially done).
pub fn is_success(code: u32) -> bool {
    matches!(code, PLACED | DONE | DONE_PARTIAL)
}

/// No verdict came back: the order may still be on the book.
pub fn is_ambiguous(code: u32) -> bool {
    matches!(code, ERROR | TIMEOUT | CONNECTION)
}

/// The MQL5 name of a retcode, or `UNKNOWN` for one this build does not
/// list. Retcode 0, which is how `order_check` reports success, is `OK`.
pub fn name(code: u32) -> &'static str {
    match code {
        0 => "OK",
        REQUOTE => "REQUOTE",
        REJECT => "REJECT",
        CANCEL => "CANCEL",
        PLACED => "PLACED",
        DONE => "DONE",
        DONE_PARTIAL => "DONE_PARTIAL",
        ERROR => "ERROR",
        TIMEOUT => "TIMEOUT",
        INVALID => "INVALID",
        INVALID_VOLUME => "INVALID_VOLUME",
        INVALID_PRICE => "INVALID_PRICE",
        INVALID_STOPS => "INVALID_STOPS",
        TRADE_DISABLED => "TRADE_DISABLED",
        MARKET_CLOSED => "MARKET_CLOSED",
        NO_MONEY => "NO_MONEY",
        PRICE_CHANGED => "PRICE_CHANGED",
        PRICE_OFF => "PRICE_OFF",
        INVALID_EXPIRATION => "INVALID_EXPIRATION",
        ORDER_CHANGED => "ORDER_CHANGED",
        TOO_MANY_REQUESTS => "TOO_MANY_REQUESTS",
        NO_CHANGES => "NO_CHANGES",
        SERVER_DISABLES_AT => "SERVER_DISABLES_AT",
        CLIENT_DISABLES_AT => "CLIENT_DISABLES_AT",
        LOCKED => "LOCKED",
        FROZEN => "FROZEN",
        INVALID_FILL => "INVALID_FILL",
        CONNECTION => "CONNECTION",
        ONLY_REAL => "ONLY_REAL",
        LIMIT_ORDERS => "LIMIT_ORDERS",
        LIMIT_VOLUME => "LIMIT_VOLUME",
        INVALID_ORDER => "INVALID_ORDER",
        POSITION_CLOSED => "POSITION_CLOSED",
        INVALID_CLOSE_VOLUME => "INVALID_CLOSE_VOLUME",
        CLOSE_ORDER_EXIST => "CLOSE_ORDER_EXIST",
        LIMIT_POSITIONS => "LIMIT_POSITIONS",
        REJECT_CANCEL => "REJECT_CANCEL",
        LONG_ONLY => "LONG_ONLY",
        SHORT_ONLY => "SHORT_ONLY",
        CLOSE_ONLY => "CLOSE_ONLY",
        FIFO_CLOSE => "FIFO_CLOSE",
        HEDGE_PROHIBITED => "HEDGE_PROHIBITED",
        _ => "UNKNOWN",
    }
}
