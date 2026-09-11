//! `MqlTradeRequest`: what an order asks the server to do.

use crate::types::{trade_action, Order, Position};

/// Zero is the terminal's own "unset" for every field, so there are no
/// options: leave what you do not need at its default.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TradeRequest {
    pub action: u32,
    pub magic: u64,
    pub order: u64,
    pub symbol: String,
    pub volume: f64,
    pub price: f64,
    pub stoplimit: f64,
    pub sl: f64,
    pub tp: f64,
    pub deviation: u64,
    pub order_type: u32,
    pub type_filling: u32,
    pub type_time: u32,
    pub expiration: i64,
    pub comment: String,
    pub position: u64,
    pub position_by: u64,
}

impl TradeRequest {
    /// A market order at `price` (the ask for a buy, the bid for a sell).
    pub fn market(symbol: &str, order_type: u32, volume: f64, price: f64) -> Self {
        TradeRequest {
            action: trade_action::DEAL,
            symbol: symbol.to_string(),
            volume,
            order_type,
            price,
            ..Default::default()
        }
    }

    /// A pending order resting at `price`.
    pub fn pending(symbol: &str, order_type: u32, volume: f64, price: f64) -> Self {
        TradeRequest {
            action: trade_action::PENDING,
            ..Self::market(symbol, order_type, volume, price)
        }
    }

    /// Close a position in full at `price`.
    ///
    /// Use the side the position is closed on: the bid for a long, the ask
    /// for a short.
    pub fn close(position: &Position, price: f64) -> Self {
        Self::close_part(position, price, position.volume)
    }

    /// Close part of a position at `price`.
    ///
    /// The volume is clamped to what is open. What remains keeps the same
    /// ticket and its stop and target.
    pub fn close_part(position: &Position, price: f64, volume: f64) -> Self {
        TradeRequest {
            action: trade_action::DEAL,
            symbol: position.symbol.clone(),
            volume: volume.clamp(0.0, position.volume),
            order_type: position.closing_order_type(),
            price,
            position: position.ticket,
            ..Default::default()
        }
    }

    /// Change an open position's stop and target.
    ///
    /// The request starts from the levels the position already has, so
    /// changing one leaves the other alone. The action replaces both on the
    /// server, so removing one is [`TradeRequest::clear_sl`] or
    /// [`TradeRequest::clear_tp`] rather than leaving it out.
    pub fn protect(position: &Position) -> Self {
        TradeRequest {
            action: trade_action::SLTP,
            symbol: position.symbol.clone(),
            position: position.ticket,
            sl: position.sl,
            tp: position.tp,
            ..Default::default()
        }
    }

    /// Move a pending order to `price`.
    ///
    /// Everything the action would otherwise replace is carried over from the
    /// order: its type, its stop and target, its expiry. Change what you mean
    /// to change and leave the rest.
    ///
    /// The type never changes, so an order dragged to the wrong side of the
    /// market is refused by the server rather than quietly becoming a
    /// different kind of order.
    pub fn modify(order: &Order, price: f64) -> Self {
        TradeRequest {
            action: trade_action::MODIFY,
            symbol: order.symbol.clone(),
            order: order.ticket,
            order_type: order.kind as u32,
            price,
            sl: order.sl,
            tp: order.tp,
            stoplimit: order.price_stoplimit,
            type_time: order.type_time as u32,
            expiration: order.time_expiration,
            ..Default::default()
        }
    }

    /// Cancel a pending order.
    pub fn remove(order: u64) -> Self {
        TradeRequest {
            action: trade_action::REMOVE,
            order,
            ..Default::default()
        }
    }

    /// Stamp an identifier the server stores and history reports back. Give
    /// every order its own: it is the only way to find out what happened to a
    /// send that failed with [`crate::Error::OutcomeUnknown`].
    pub fn magic(mut self, magic: u64) -> Self {
        self.magic = magic;
        self
    }
    /// Slippage budget in points for a market order. A budget of zero is
    /// rejected outright by brokers that honour it.
    pub fn deviation(mut self, points: u64) -> Self {
        self.deviation = points;
        self
    }
    /// Put the stop loss at `price`.
    ///
    /// A stop is independent of a target: setting one does not require the
    /// other. The level must clear [`crate::SymbolInfo::min_stop_distance`]
    /// from the market, or the server answers `INVALID_STOPS`, and it should
    /// be on the contract's grid, which is what
    /// [`crate::SymbolInfo::normalize_price`] is for.
    pub fn sl(mut self, price: f64) -> Self {
        self.sl = price;
        self
    }

    /// Put the take profit at `price`. Independent of the stop, with the same
    /// distance and grid rules.
    pub fn tp(mut self, price: f64) -> Self {
        self.tp = price;
        self
    }

    /// Remove the stop loss.
    ///
    /// Zero is how the protocol spells "no level", so this is the same thing
    /// as `sl(0.0)` said out loud.
    pub fn clear_sl(self) -> Self {
        self.sl(0.0)
    }

    /// Remove the take profit.
    pub fn clear_tp(self) -> Self {
        self.tp(0.0)
    }
    /// Set the filling mode. Not optional in practice — see
    /// [`crate::SymbolInfo::preferred_filling`].
    pub fn filling(mut self, mode: u32) -> Self {
        self.type_filling = mode;
        self
    }
    /// A note carried with the order; the terminal shows it in the journal
    /// and history keeps it. Truncated to 31 characters on the wire.
    pub fn comment(mut self, comment: &str) -> Self {
        self.comment = comment.to_string();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{order_type, position_type};

    fn long() -> Position {
        Position {
            ticket: 11,
            symbol: "EURUSD".to_string(),
            kind: position_type::BUY,
            volume: 1.0,
            sl: 1.0900,
            tp: 1.1200,
            ..Default::default()
        }
    }

    #[test]
    fn a_close_never_takes_more_than_is_open() {
        let held = long();
        assert_eq!(TradeRequest::close_part(&held, 100.0, 5.0).volume, 1.0);
        assert_eq!(TradeRequest::close_part(&held, 100.0, 0.4).volume, 0.4);
        assert_eq!(TradeRequest::close_part(&held, 100.0, -1.0).volume, 0.0);
        let full = TradeRequest::close(&held, 100.0);
        assert_eq!(full.volume, 1.0);
        assert_eq!(
            full.order_type,
            order_type::SELL,
            "a long is closed by selling"
        );
        assert_eq!(full.position, 11);
    }

    /// The trap this shape exists to remove: the SLTP action replaces both
    /// levels, so a request built from nothing wipes whichever one the caller
    /// did not mention.
    #[test]
    fn changing_one_level_leaves_the_other_where_it_was() {
        let held = long();

        let moved_stop = TradeRequest::protect(&held).sl(1.1000);
        assert_eq!(moved_stop.sl, 1.1000);
        assert_eq!(moved_stop.tp, 1.1200, "the target was not being changed");

        let moved_target = TradeRequest::protect(&held).tp(1.1500);
        assert_eq!(moved_target.sl, 1.0900);
        assert_eq!(moved_target.tp, 1.1500);

        // Removing one is said out loud, not implied by omission.
        let no_target = TradeRequest::protect(&held).clear_tp();
        assert_eq!(no_target.sl, 1.0900);
        assert_eq!(no_target.tp, 0.0);
    }

    #[test]
    fn a_stop_and_a_target_are_independent_of_each_other() {
        let entry = || TradeRequest::market("EURUSD", order_type::BUY, 0.1, 1.1000);
        let stop_only = entry().sl(1.0900);
        assert_eq!(stop_only.sl, 1.0900);
        assert_eq!(stop_only.tp, 0.0, "no target is zero, not a requirement");

        let target_only = entry().tp(1.1200);
        assert_eq!(target_only.sl, 0.0);
        assert_eq!(target_only.tp, 1.1200);

        let both = entry().sl(1.0900).tp(1.1200);
        assert_eq!((both.sl, both.tp), (1.0900, 1.1200));
    }

    /// A moved order keeps its type and everything the action would replace.
    #[test]
    fn a_moved_order_carries_its_levels_and_its_kind() {
        let resting = Order {
            ticket: 42,
            symbol: "EURUSD".to_string(),
            kind: order_type::BUY_LIMIT as i32,
            price_open: 1.0800,
            sl: 1.0700,
            tp: 1.0950,
            type_time: 2,
            time_expiration: 1_700_000_000,
            ..Default::default()
        };
        let moved = TradeRequest::modify(&resting, 1.0850);
        assert_eq!(moved.price, 1.0850);
        assert_eq!(moved.order, 42);
        assert_eq!(
            moved.order_type,
            order_type::BUY_LIMIT,
            "the kind never changes"
        );
        assert_eq!((moved.sl, moved.tp), (1.0700, 1.0950));
        assert_eq!(moved.type_time, 2);
        assert_eq!(moved.expiration, 1_700_000_000);
    }
}
