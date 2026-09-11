//! What an order is, how it fills, and how long it lives.

wire_enum! {
    /// `ENUM_ORDER_TYPE`. Which way, and at market or waiting at a price.
    ///
    /// Which pending kind a price means is arithmetic, not a preference:
    /// above the market a buy is a stop and a sell is a limit, below it the
    /// other way round.
    OrderType: i32 {
        Buy = 0, "BUY";
        Sell = 1, "SELL";
        /// Waits below the market for the price to come back down.
        BuyLimit = 2, "BUY_LIMIT";
        /// Waits above the market for the price to come back up.
        SellLimit = 3, "SELL_LIMIT";
        /// Waits above the market for the price to run on.
        BuyStop = 4, "BUY_STOP";
        /// Waits below the market for the price to run on.
        SellStop = 5, "SELL_STOP";
        BuyStopLimit = 6, "BUY_STOP_LIMIT";
        SellStopLimit = 7, "SELL_STOP_LIMIT";
        CloseBy = 8, "CLOSE_BY";
    }
}

impl OrderType {
    /// Whether this waits at a price rather than trading now.
    pub const fn is_pending(self) -> bool {
        !matches!(self, OrderType::Buy | OrderType::Sell | OrderType::CloseBy)
    }
}

wire_enum! {
    /// `ENUM_ORDER_TYPE_FILLING`: what to do when the book cannot fill the
    /// whole volume.
    ///
    /// Not optional in practice. The default is `Fok`, and a broker that does
    /// not offer it answers [`crate::RetCode::InvalidFill`] before looking at
    /// anything else. See [`crate::SymbolInfo::preferred_filling`].
    OrderFilling: i32 {
        /// All of it at once, or nothing.
        Fok = 0, "FOK";
        /// As much as the book holds now; cancel the rest.
        Ioc = 1, "IOC";
        /// As much as the book holds now; leave the rest resting.
        Return = 2, "RETURN";
        /// Passive only: cancel rather than take liquidity.
        Boc = 3, "BOC";
    }
}

wire_enum! {
    /// `ENUM_ORDER_TYPE_TIME`: how long a pending order lives.
    OrderTime: i32 {
        /// Until cancelled.
        Gtc = 0, "GTC";
        /// Until the end of the trading day.
        Day = 1, "DAY";
        /// Until the moment in `expiration`.
        Specified = 2, "SPECIFIED";
        /// Until the end of the day in `expiration`.
        SpecifiedDay = 3, "SPECIFIED_DAY";
    }
}

wire_enum! {
    /// `ENUM_ORDER_STATE`: where an order stands.
    OrderState: i32 {
        Started = 0, "STARTED";
        Placed = 1, "PLACED";
        Canceled = 2, "CANCELED";
        Partial = 3, "PARTIAL";
        Filled = 4, "FILLED";
        Rejected = 5, "REJECTED";
        Expired = 6, "EXPIRED";
        RequestAdd = 7, "REQUEST_ADD";
        RequestModify = 8, "REQUEST_MODIFY";
        RequestCancel = 9, "REQUEST_CANCEL";
    }
}

wire_enum! {
    /// `ENUM_TRADE_REQUEST_ACTIONS`: what a [`crate::TradeRequest`] asks for.
    TradeAction: i32 {
        /// Trade now at market.
        Deal = 1, "DEAL";
        /// Place an order that waits at a price.
        Pending = 5, "PENDING";
        /// Change an open position's stop and target.
        Sltp = 6, "SLTP";
        /// Change a pending order.
        Modify = 7, "MODIFY";
        /// Cancel a pending order.
        Remove = 8, "REMOVE";
        /// Close one position against an opposite one.
        CloseBy = 10, "CLOSE_BY";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_pending_kinds_know_they_wait() {
        for waiting in [
            OrderType::BuyLimit,
            OrderType::SellLimit,
            OrderType::BuyStop,
            OrderType::SellStop,
        ] {
            assert!(waiting.is_pending(), "{waiting}");
        }
        assert!(!OrderType::Buy.is_pending());
        assert!(!OrderType::Sell.is_pending());
    }

    #[test]
    fn zero_decodes_to_what_the_protocol_numbers_zero() {
        assert_eq!(OrderType::default(), OrderType::Buy);
        assert_eq!(OrderFilling::default(), OrderFilling::Fok);
        assert_eq!(OrderTime::default(), OrderTime::Gtc);
        assert_eq!(OrderState::default(), OrderState::Started);
        // Actions start at one, so a zeroed request has no action yet.
        assert_eq!(TradeAction::default(), TradeAction::Unknown(0));
    }
}
