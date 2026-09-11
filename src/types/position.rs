//! Open positions, orders on the book, and the deals that made them.

use crate::types::{order_type, position_type};

/// An open position: one net exposure on one symbol, with the money it is
/// worth right now.
#[derive(Debug, Clone, Default)]
pub struct Position {
    pub ticket: u64,
    pub time: i64,
    pub time_msc: i64,
    pub time_update: i64,
    pub time_update_msc: i64,
    /// [`position_type`].
    pub kind: i32,
    pub magic: u64,
    pub identifier: u64,
    pub reason: i32,
    pub volume: f64,
    pub price_open: f64,
    pub sl: f64,
    pub tp: f64,
    pub price_current: f64,
    pub swap: f64,
    pub profit: f64,
    pub symbol: String,
    pub comment: String,
    pub external_id: String,
}

impl Position {
    /// The order type that closes this position.
    pub fn closing_order_type(&self) -> u32 {
        if self.kind == position_type::BUY {
            order_type::SELL
        } else {
            order_type::BUY
        }
    }
}

/// An order — resting on the book, or finished and in history. The state
/// field says which.
#[derive(Debug, Clone, Default)]
pub struct Order {
    pub ticket: u64,
    pub time_setup: i64,
    pub time_setup_msc: i64,
    pub time_done: i64,
    pub time_done_msc: i64,
    pub time_expiration: i64,
    /// [`order_type`].
    pub kind: i32,
    pub type_time: i32,
    pub type_filling: i32,
    pub state: i32,
    pub magic: u64,
    pub position_id: u64,
    pub position_by_id: u64,
    pub reason: i32,
    pub volume_initial: f64,
    pub volume_current: f64,
    pub price_open: f64,
    pub price_current: f64,
    pub sl: f64,
    pub tp: f64,
    pub price_stoplimit: f64,
    pub symbol: String,
    pub comment: String,
    pub external_id: String,
}

/// A deal: one execution that actually happened, with what it cost. Deals
/// are what a position is built from, and the only place profit is final.
#[derive(Debug, Clone, Default)]
pub struct Deal {
    pub ticket: u64,
    pub order: u64,
    pub time: i64,
    pub time_msc: i64,
    /// `deal_type`.
    pub kind: i32,
    /// `deal_entry`.
    pub entry: i32,
    pub magic: u64,
    pub position_id: u64,
    pub reason: i32,
    pub volume: f64,
    pub price: f64,
    pub commission: f64,
    pub swap: f64,
    pub profit: f64,
    pub fee: f64,
    pub symbol: String,
    pub comment: String,
    pub external_id: String,
}
