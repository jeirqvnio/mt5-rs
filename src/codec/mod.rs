//! Record layouts. The field order here *is* the protocol: each decoder is a
//! table read top to bottom, every field consumed (skipped ones included),
//! and the cursor must land exactly at the end of the body.
//!
//! Layouts were established by observing the official Python module against
//! terminal builds 5836–6140. A build that moves a field shows up as a
//! decode error naming the field, not as a wrong number.

#[macro_use]
mod fields;
mod account;
mod market;
mod position;
mod symbol;
mod trade;

pub use account::{account_info, terminal_info, version};
pub use fields::count;
pub use market::{book, rates, tick_one, ticks};
pub use position::{deal_one, deals, order_one, orders, position_one, positions};
pub use symbol::{symbol_one, symbols};
pub use trade::{check_result, trade_request, trade_result};

/// Fixed-width UTF-16LE string slots shared by several records, in bytes.
pub(crate) const SLOT_SYMBOL: usize = 64;
pub(crate) const SLOT_COMMENT: usize = 64;
pub(crate) const SLOT_EXTERNAL_ID: usize = 64;
