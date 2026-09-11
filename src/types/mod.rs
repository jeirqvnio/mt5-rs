//! The MetaTrader 5 data model, field for field as the terminal sends it.
//!
//! MQL5 enumerations are Rust enums that name themselves, convert both ways
//! and keep a value they do not recognise rather than losing it. The two real
//! bitmasks stay masks. Every timestamp is broker server time, Unix seconds
//! (`*_msc`: milliseconds); the terminal never reports the server's UTC
//! offset, so take `symbol_tick().time` as the reference clock.

#[macro_use]
mod code;

mod account;
mod deal_kind;
mod flags;
mod market;
mod order_kind;
mod position;
mod request;
mod result;
mod retcode;
mod symbol;
mod timeframe;

pub use account::{AccountInfo, AccountTradeMode, TerminalInfo, TerminalVersion};
pub use deal_kind::{DealEntry, DealType, PositionType};
pub use flags::{copy_ticks, filling_mask};
pub use market::{BookEntry, Rate, Tick};
pub use order_kind::{OrderFilling, OrderState, OrderTime, OrderType, TradeAction};
pub use position::{Deal, Order, Position};
pub use request::TradeRequest;
pub use result::{TradeCheckResult, TradeResult};
pub use retcode::RetCode;
pub use symbol::{SymbolInfo, SymbolTradeMode};
pub use timeframe::Timeframe;
