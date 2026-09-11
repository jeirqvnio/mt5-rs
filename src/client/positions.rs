//! What is open now, and what history says.

use crate::client::{commands, Mt5};
use crate::codec;
use crate::error::Result;
use crate::types::{Deal, Order, Position};
use crate::wire::Writer;

fn range(from: i64, to: i64) -> Vec<u8> {
    Writer::new().i64(from).i64(to).into_bytes()
}

fn ticket(id: u64) -> Vec<u8> {
    Writer::new().u64(id).into_bytes()
}

impl Mt5 {
    /// How many positions are open, without decoding any of them.
    ///
    /// One small reply where [`Mt5::positions`] costs a 320-byte record each.
    pub async fn positions_total(&self) -> Result<u32> {
        self.call_u32("positions_total", commands::POSITIONS_TOTAL, &[])
            .await
    }

    /// Every open position, or only those on one symbol.
    ///
    /// The symbol filter is served by the terminal, so it costs no more than
    /// the unfiltered call.
    pub async fn positions(&self, symbol: Option<&str>) -> Result<Vec<Position>> {
        let raw = match symbol {
            Some(s) => {
                let body = Writer::new().string(s).into_bytes();
                self.call("positions", commands::POSITIONS_GET_SYMBOL, &body)
                    .await?
            }
            None => self.call("positions", commands::POSITIONS_GET, &[]).await?,
        };
        codec::positions(&raw)
    }

    /// One open position by ticket, or `None` when no such position is open.
    ///
    /// A closed position is not here — look in [`Mt5::history_deals_of_position`].
    pub async fn position(&self, id: u64) -> Result<Option<Position>> {
        self.call_opt(
            "position",
            commands::POSITIONS_GET_TICKET,
            &ticket(id),
            codec::position_one,
        )
        .await
    }

    /// How many orders are resting on the book, without decoding any of them.
    pub async fn orders_total(&self) -> Result<u32> {
        self.call_u32("orders_total", commands::ORDERS_TOTAL, &[])
            .await
    }

    /// Every pending order, or only those on one symbol.
    ///
    /// Only orders still waiting; a filled or cancelled one has moved to
    /// [`Mt5::history_orders`].
    pub async fn orders(&self, symbol: Option<&str>) -> Result<Vec<Order>> {
        let raw = match symbol {
            Some(s) => {
                let body = Writer::new().string(s).into_bytes();
                self.call("orders", commands::ORDERS_GET_SYMBOL, &body)
                    .await?
            }
            None => self.call("orders", commands::ORDERS_GET, &[]).await?,
        };
        codec::orders(&raw)
    }

    /// One pending order by ticket, or `None` when it is no longer resting.
    pub async fn order(&self, id: u64) -> Result<Option<Order>> {
        self.call_opt(
            "order",
            commands::ORDERS_GET_TICKET,
            &ticket(id),
            codec::order_one,
        )
        .await
    }

    /// How many orders the history holds between two times, without decoding
    /// them. Both bounds are broker server time, Unix seconds.
    pub async fn history_orders_total(&self, from: i64, to: i64) -> Result<u32> {
        self.call_u32(
            "history_orders_total",
            commands::HISTORY_ORDERS_TOTAL,
            &range(from, to),
        )
        .await
    }

    /// Orders that have finished — filled, cancelled or rejected — between two
    /// times. Both bounds are broker server time, Unix seconds.
    ///
    /// This is where a guarded send looks for its `magic` after an
    /// [`crate::Error::OutcomeUnknown`].
    pub async fn history_orders(&self, from: i64, to: i64) -> Result<Vec<Order>> {
        codec::orders(
            &self
                .call(
                    "history_orders",
                    commands::HISTORY_ORDERS_GET,
                    &range(from, to),
                )
                .await?,
        )
    }

    /// Every historical order belonging to one position, without a time range
    /// to guess at. The terminal does the matching.
    pub async fn history_orders_of_position(&self, position: u64) -> Result<Vec<Order>> {
        let raw = self
            .call(
                "history_orders_of_position",
                commands::HISTORY_ORDERS_GET_POSITION,
                &ticket(position),
            )
            .await?;
        codec::orders(&raw)
    }

    /// One finished order by ticket, or `None` when the history has no such
    /// ticket.
    pub async fn history_order(&self, id: u64) -> Result<Option<Order>> {
        self.call_opt(
            "history_order",
            commands::HISTORY_ORDERS_GET_TICKET,
            &ticket(id),
            codec::order_one,
        )
        .await
    }

    /// How many deals the history holds between two times, without decoding
    /// them. Both bounds are broker server time, Unix seconds.
    pub async fn history_deals_total(&self, from: i64, to: i64) -> Result<u32> {
        self.call_u32(
            "history_deals_total",
            commands::HISTORY_DEALS_TOTAL,
            &range(from, to),
        )
        .await
    }

    /// Deals — the actual executions, with their profit, commission and swap —
    /// between two times, in broker server time, Unix seconds.
    pub async fn history_deals(&self, from: i64, to: i64) -> Result<Vec<Deal>> {
        codec::deals(
            &self
                .call(
                    "history_deals",
                    commands::HISTORY_DEALS_GET,
                    &range(from, to),
                )
                .await?,
        )
    }

    /// Every deal belonging to one position: what opened it, what closed it,
    /// and what each leg cost. The call reconciliation wants.
    pub async fn history_deals_of_position(&self, position: u64) -> Result<Vec<Deal>> {
        let raw = self
            .call(
                "history_deals_of_position",
                commands::HISTORY_DEALS_GET_POSITION,
                &ticket(position),
            )
            .await?;
        codec::deals(&raw)
    }

    /// One deal by ticket, or `None` when the history has no such ticket.
    pub async fn history_deal(&self, id: u64) -> Result<Option<Deal>> {
        self.call_opt(
            "history_deal",
            commands::HISTORY_DEALS_GET_TICKET,
            &ticket(id),
            codec::deal_one,
        )
        .await
    }
}
