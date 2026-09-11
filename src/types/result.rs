//! What the server answers a trade request with.

use crate::types::RetCode;

/// `MqlTradeResult`.
#[derive(Debug, Clone, Default)]
pub struct TradeResult {
    pub retcode: RetCode,
    pub deal: u64,
    pub order: u64,
    pub volume: f64,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub comment: String,
    pub request_id: u32,
    pub retcode_external: i32,
}

impl TradeResult {
    /// Whether the server took the order: placed, done, or partially done.
    pub const fn is_success(&self) -> bool {
        self.retcode.is_success()
    }
}

/// `MqlTradeCheckResult`. Success is retcode **0** here, not `DONE`.
#[derive(Debug, Clone, Default)]
pub struct TradeCheckResult {
    pub retcode: RetCode,
    pub balance: f64,
    pub equity: f64,
    pub profit: f64,
    pub margin: f64,
    pub margin_free: f64,
    pub margin_level: f64,
    pub comment: String,
}

impl TradeCheckResult {
    /// Whether the server found the request valid. `order_check` reports
    /// success as retcode 0, not `DONE`, which is why this exists.
    pub const fn is_ok(&self) -> bool {
        matches!(self.retcode, RetCode::Ok)
    }
}
