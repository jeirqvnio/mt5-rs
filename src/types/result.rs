//! What the server answers a trade request with.

use crate::types::retcode;

/// `MqlTradeResult`.
#[derive(Debug, Clone, Default)]
pub struct TradeResult {
    /// [`retcode`].
    pub retcode: u32,
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
    pub fn is_success(&self) -> bool {
        retcode::is_success(self.retcode)
    }
}

/// `MqlTradeCheckResult`. Success is retcode **0** here, not `DONE`.
#[derive(Debug, Clone, Default)]
pub struct TradeCheckResult {
    pub retcode: u32,
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
    pub fn is_ok(&self) -> bool {
        self.retcode == 0
    }
}
