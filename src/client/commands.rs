//! IPC command codes, established by observing the official Python module.
//!
//! A filtered read is its own command: `X0` count, `X1` all, `X2` by string,
//! `X3` by ticket. Command 3 is *not* `last_error` — the terminal closes the
//! pipe on it.

pub const VERSION: u32 = 1;
pub const SESSION_HELLO: u32 = 3;
pub const SESSION: u32 = 4;
pub const LOGIN: u32 = 5;

pub const COPY_TICKS_FROM: u32 = 104;
pub const COPY_TICKS_RANGE: u32 = 105;
pub const COPY_RATES_FROM: u32 = 106;
pub const COPY_RATES_RANGE: u32 = 107;
pub const COPY_RATES_FROM_POS: u32 = 108;

pub const POSITIONS_TOTAL: u32 = 120;
pub const POSITIONS_GET: u32 = 121;
pub const POSITIONS_GET_SYMBOL: u32 = 122;
pub const POSITIONS_GET_TICKET: u32 = 123;
pub const ORDERS_TOTAL: u32 = 130;
pub const ORDERS_GET: u32 = 131;
pub const ORDERS_GET_SYMBOL: u32 = 132;
pub const ORDERS_GET_TICKET: u32 = 133;
pub const HISTORY_ORDERS_TOTAL: u32 = 140;
pub const HISTORY_ORDERS_GET: u32 = 141;
pub const HISTORY_ORDERS_GET_POSITION: u32 = 142;
pub const HISTORY_ORDERS_GET_TICKET: u32 = 143;
pub const HISTORY_DEALS_TOTAL: u32 = 150;
pub const HISTORY_DEALS_GET: u32 = 151;
pub const HISTORY_DEALS_GET_POSITION: u32 = 152;
pub const HISTORY_DEALS_GET_TICKET: u32 = 153;

pub const ORDER_CHECK: u32 = 160;
pub const ORDER_SEND: u32 = 161;

pub const SYMBOL_INFO: u32 = 170;
pub const SYMBOL_SELECT: u32 = 171;
pub const SYMBOL_INFO_TICK: u32 = 172;
pub const SYMBOLS_TOTAL: u32 = 173;
pub const SYMBOLS_GET: u32 = 174;
pub const SYMBOLS_GET_GROUP: u32 = 175;

pub const TERMINAL_INFO: u32 = 180;
pub const ACCOUNT_INFO: u32 = 190;
pub const MARKET_BOOK_ADD: u32 = 191;
pub const MARKET_BOOK_RELEASE: u32 = 192;
pub const MARKET_BOOK_GET: u32 = 193;

/// Read out of the `MetaTrader5` 5.0.6090 module itself rather than guessed:
/// each function loads its number as an immediate before the send, and in the
/// disassembly the two sit in one block right after `order_check` 0xa0 and
/// `order_send` 0xa1. These were 202 and 203, which the terminal answered with
/// `unknown IPC command` every time.
pub const ORDER_CALC_MARGIN: u32 = 162;
pub const ORDER_CALC_PROFIT: u32 = 163;
