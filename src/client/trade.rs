//! The calls that reach the trade server.

use crate::client::{commands, Mt5};
use crate::codec;
use crate::error::Result;
use crate::types::{OrderType, TradeCheckResult, TradeRequest, TradeResult};
use crate::wire::{Cursor, Writer};

impl Mt5 {
    /// Margin the server would require, in account currency. Equals the
    /// `margin` an `order_check` for the same request reports.
    pub async fn calc_margin(
        &self,
        order_type: OrderType,
        symbol: &str,
        volume: f64,
        price: f64,
    ) -> Result<f64> {
        let body = Writer::new()
            .i32(order_type.code())
            .string(symbol)
            .f64(volume)
            .f64(price)
            .into_bytes();
        Cursor::new(
            &self
                .call("calc_margin", commands::ORDER_CALC_MARGIN, &body)
                .await?,
        )
        .f64("margin")
    }

    /// Profit of `volume` between two prices, in account currency.
    pub async fn calc_profit(
        &self,
        order_type: OrderType,
        symbol: &str,
        volume: f64,
        open: f64,
        close: f64,
    ) -> Result<f64> {
        let body = Writer::new()
            .i32(order_type.code())
            .string(symbol)
            .f64(volume)
            .f64(open)
            .f64(close)
            .into_bytes();
        Cursor::new(
            &self
                .call("calc_profit", commands::ORDER_CALC_PROFIT, &body)
                .await?,
        )
        .f64("profit")
    }

    /// Validate a request against the server without placing anything.
    pub async fn order_check(&self, request: &TradeRequest) -> Result<TradeCheckResult> {
        let body = codec::trade_request(request)?;
        codec::check_result(
            &self
                .call_once("order_check", commands::ORDER_CHECK, &body)
                .await?,
        )
    }

    /// Send a trade request. **Never retried.** A transport failure is
    /// `Error::OutcomeUnknown`: the order may be on the book, so look at
    /// `orders`/`history_orders` (by magic) before sending again.
    pub async fn order_send(&self, request: &TradeRequest) -> Result<TradeResult> {
        let body = codec::trade_request(request)?;
        codec::trade_result(
            &self
                .call_once("order_send", commands::ORDER_SEND, &body)
                .await?,
        )
    }
}
