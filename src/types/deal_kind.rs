//! What a deal was, and what it did to a position.

wire_enum! {
    /// `ENUM_DEAL_TYPE`. Most deals are a trade; the rest move money without
    /// touching the market.
    DealType: i32 {
        Buy = 0, "BUY";
        Sell = 1, "SELL";
        /// A deposit or a withdrawal.
        Balance = 2, "BALANCE";
        Credit = 3, "CREDIT";
        Charge = 4, "CHARGE";
        Correction = 5, "CORRECTION";
        Bonus = 6, "BONUS";
        Commission = 7, "COMMISSION";
    }
}

wire_enum! {
    /// `ENUM_DEAL_ENTRY`: which side of a position's life this deal is.
    DealEntry: i32 {
        /// Opened it, or added to it.
        In = 0, "IN";
        /// Closed it, or took some off.
        Out = 1, "OUT";
        /// Closed it and opened the opposite in one go.
        InOut = 2, "INOUT";
        /// Closed by an opposite position.
        OutBy = 3, "OUT_BY";
    }
}

wire_enum! {
    /// `ENUM_POSITION_TYPE`: which way a position faces.
    PositionType: i32 {
        Buy = 0, "BUY";
        Sell = 1, "SELL";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_deal_names_itself() {
        assert_eq!(DealType::Balance.to_string(), "BALANCE");
        assert_eq!(DealEntry::Out.to_string(), "OUT");
        assert_eq!(PositionType::Sell.to_string(), "SELL");
        assert_eq!(DealType::default(), DealType::Buy);
        assert_eq!(DealEntry::default(), DealEntry::In);
        assert_eq!(PositionType::default(), PositionType::Buy);
    }
}
