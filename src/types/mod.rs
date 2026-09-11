//! The MetaTrader 5 data model, field for field as the terminal sends it.
//!
//! Integer types are wire widths. Enumerations are plain integers with
//! constant modules, so a value from a newer build round-trips instead of
//! failing. Every timestamp is broker server time, Unix seconds (`*_msc`:
//! milliseconds); the terminal never reports the server's UTC offset, so take
//! `symbol_tick().time` as the reference clock.

mod account;
mod consts;
mod market;
mod position;
mod request;
mod result;
pub mod retcode;
mod symbol;
mod timeframe;

pub use account::{AccountInfo, TerminalInfo, TerminalVersion};
pub use consts::{
    copy_ticks, deal_entry, deal_type, filling_mask, order_filling, order_time, order_type,
    position_type, trade_action,
};
pub use market::{BookEntry, Rate, Tick};
pub use position::{Deal, Order, Position};
pub use request::TradeRequest;
pub use result::{TradeCheckResult, TradeResult};
pub use symbol::SymbolInfo;
pub use timeframe::Timeframe;
