//! `MqlTradeResult::retcode`: what the trade server made of a request.

wire_enum! {
    /// The trade server's verdict.
    ///
    /// `order_check` reports a valid request as [`RetCode::Ok`]; `order_send`
    /// reports a filled one as [`RetCode::Done`]. That difference is why
    /// [`crate::TradeCheckResult::is_ok`] and
    /// [`crate::TradeResult::is_success`] are not the same test.
    RetCode: u32 {
        /// `order_check` found nothing wrong. Never returned by `order_send`.
        Ok = 0, "OK";
        Requote = 10004, "REQUOTE";
        Reject = 10006, "REJECT";
        Cancel = 10007, "CANCEL";
        /// A pending order is on the book.
        Placed = 10008, "PLACED";
        /// The deal went through.
        Done = 10009, "DONE";
        /// Part of the volume went through; the rest did not.
        DonePartial = 10010, "DONE_PARTIAL";
        Error = 10011, "ERROR";
        Timeout = 10012, "TIMEOUT";
        Invalid = 10013, "INVALID";
        InvalidVolume = 10014, "INVALID_VOLUME";
        InvalidPrice = 10015, "INVALID_PRICE";
        /// A level is nearer the market than the broker allows, or on the
        /// wrong side of it. See [`crate::SymbolInfo::min_stop_distance`].
        InvalidStops = 10016, "INVALID_STOPS";
        TradeDisabled = 10017, "TRADE_DISABLED";
        MarketClosed = 10018, "MARKET_CLOSED";
        NoMoney = 10019, "NO_MONEY";
        PriceChanged = 10020, "PRICE_CHANGED";
        PriceOff = 10021, "PRICE_OFF";
        InvalidExpiration = 10022, "INVALID_EXPIRATION";
        OrderChanged = 10023, "ORDER_CHANGED";
        TooManyRequests = 10024, "TOO_MANY_REQUESTS";
        NoChanges = 10025, "NO_CHANGES";
        ServerDisablesAt = 10026, "SERVER_DISABLES_AT";
        /// Algo trading is switched off in the terminal.
        ClientDisablesAt = 10027, "CLIENT_DISABLES_AT";
        Locked = 10028, "LOCKED";
        Frozen = 10029, "FROZEN";
        /// The contract does not offer the filling mode that was asked for.
        /// See [`crate::SymbolInfo::preferred_filling`].
        InvalidFill = 10030, "INVALID_FILL";
        Connection = 10031, "CONNECTION";
        OnlyReal = 10032, "ONLY_REAL";
        LimitOrders = 10033, "LIMIT_ORDERS";
        LimitVolume = 10034, "LIMIT_VOLUME";
        InvalidOrder = 10035, "INVALID_ORDER";
        PositionClosed = 10036, "POSITION_CLOSED";
        InvalidCloseVolume = 10038, "INVALID_CLOSE_VOLUME";
        CloseOrderExist = 10039, "CLOSE_ORDER_EXIST";
        LimitPositions = 10040, "LIMIT_POSITIONS";
        RejectCancel = 10041, "REJECT_CANCEL";
        LongOnly = 10042, "LONG_ONLY";
        ShortOnly = 10043, "SHORT_ONLY";
        CloseOnly = 10044, "CLOSE_ONLY";
        FifoClose = 10045, "FIFO_CLOSE";
        HedgeProhibited = 10046, "HEDGE_PROHIBITED";
    }
}

impl RetCode {
    /// Whether the server took the order: placed, done, or partly done.
    ///
    /// [`RetCode::Ok`] is not among them. It means a check passed, and
    /// nothing was sent.
    pub const fn is_success(self) -> bool {
        matches!(self, RetCode::Placed | RetCode::Done | RetCode::DonePartial)
    }

    /// Whether no verdict came back, so the order may or may not be on the
    /// book. Reconcile by magic before sending anything again.
    pub const fn is_ambiguous(self) -> bool {
        matches!(
            self,
            RetCode::Error | RetCode::Timeout | RetCode::Connection
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verdict_names_itself_without_a_lookup() {
        assert_eq!(RetCode::Done.to_string(), "DONE");
        assert_eq!(RetCode::from_code(10009), RetCode::Done);
        assert_eq!(RetCode::Done.code(), 10009);
    }

    #[test]
    fn a_value_from_a_newer_build_survives_the_round_trip() {
        let future = RetCode::from_code(10_099);
        assert_eq!(future, RetCode::Unknown(10_099));
        assert_eq!(future.code(), 10_099, "the number must not be lost");
        assert_eq!(future.to_string(), "UNKNOWN(10099)");
        assert!(!future.is_success(), "never guess in our own favour");
        assert!(!future.is_ambiguous());
    }

    /// The distinction the two result types turn on.
    #[test]
    fn a_passed_check_is_not_a_filled_order() {
        assert!(!RetCode::Ok.is_success());
        assert!(RetCode::Done.is_success());
        assert!(RetCode::Placed.is_success());
        assert!(RetCode::DonePartial.is_success());
    }

    #[test]
    fn only_a_missing_verdict_is_ambiguous() {
        for unsettled in [RetCode::Error, RetCode::Timeout, RetCode::Connection] {
            assert!(unsettled.is_ambiguous(), "{unsettled}");
        }
        // A clean refusal settles it: nothing was placed.
        assert!(!RetCode::NoMoney.is_ambiguous());
        assert!(!RetCode::Done.is_ambiguous());
    }
}
