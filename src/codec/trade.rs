//! `MqlTradeRequest` out, `MqlTradeResult` / `MqlTradeCheckResult` back.

use crate::codec::{SLOT_COMMENT, SLOT_SYMBOL};
use crate::error::{Error, Result};
use crate::types::{TradeCheckResult, TradeRequest, TradeResult};
use crate::wire::{Cursor, Writer};

/// The request is exactly this long; anything else means the layout moved,
/// and a wrong request does not fail — it sends a different order.
const REQUEST_BYTES: usize = 232;
const RESULT_COMMENT_SLOT: usize = 200;
const TRADE_RESULT_BYTES: usize = 260;
const CHECK_RESULT_BYTES: usize = 252;

pub fn trade_request(r: &TradeRequest) -> Result<Vec<u8>> {
    let body = Writer::new()
        .i32(r.action.code())
        .u64(r.magic)
        .u64(r.order)
        .fixed_string(&r.symbol, SLOT_SYMBOL)
        .f64(r.volume)
        .f64(r.price)
        .f64(r.stoplimit)
        .f64(r.sl)
        .f64(r.tp)
        .u64(r.deviation)
        .i32(r.order_type.code())
        .i32(r.type_filling.code())
        .i32(r.type_time.code())
        .i64(r.expiration)
        .fixed_string(&r.comment, SLOT_COMMENT)
        .u64(r.position)
        .u64(r.position_by)
        .into_bytes();
    if body.len() != REQUEST_BYTES {
        return Err(Error::Protocol(format!(
            "trade request encoded to {} bytes, expected {REQUEST_BYTES}; refusing to send",
            body.len()
        )));
    }
    Ok(body)
}

pub fn trade_result(buf: &[u8]) -> Result<TradeResult> {
    if buf.len() < TRADE_RESULT_BYTES {
        return Err(Error::Protocol(format!(
            "trade result is {} bytes, expected {TRADE_RESULT_BYTES}",
            buf.len()
        )));
    }
    let mut c = Cursor::new(buf);
    fields!(c, "result", {
        retcode: enum_u32, deal: u64, order: u64, volume: f64, price: f64, bid: f64, ask: f64,
        comment: fixed_string(RESULT_COMMENT_SLOT), request_id: u32, retcode_external: i32,
    });
    Ok(TradeResult {
        retcode,
        deal,
        order,
        volume,
        price,
        bid,
        ask,
        comment,
        request_id,
        retcode_external,
    })
}

pub fn check_result(buf: &[u8]) -> Result<TradeCheckResult> {
    if buf.len() < CHECK_RESULT_BYTES {
        return Err(Error::Protocol(format!(
            "check result is {} bytes, expected {CHECK_RESULT_BYTES}",
            buf.len()
        )));
    }
    let mut c = Cursor::new(buf);
    fields!(c, "check", {
        retcode: enum_u32, balance: f64, equity: f64, profit: f64, margin: f64, margin_free: f64,
        margin_level: f64, comment: fixed_string(RESULT_COMMENT_SLOT),
    });
    Ok(TradeCheckResult {
        retcode,
        balance,
        equity,
        profit,
        margin,
        margin_free,
        margin_level,
        comment,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::{OrderFilling, OrderTime, OrderType, TradeAction};

    /// Every field of a fully populated request, read back at the offset the
    /// terminal expects it. This pins the layout: a changed width, a reordered
    /// field or a resized slot moves something here and fails the test rather
    /// than sending a different order.
    #[test]
    fn the_request_is_232_bytes_with_every_field_where_the_terminal_expects_it() {
        let request = TradeRequest {
            action: TradeAction::Pending,
            magic: 777,
            order: 88,
            symbol: "EURUSD".to_string(),
            volume: 0.1,
            price: 1.2345,
            stoplimit: 1.2000,
            sl: 1.1000,
            tp: 1.3000,
            deviation: 20,
            order_type: OrderType::SellStopLimit,
            type_filling: OrderFilling::Boc,
            type_time: OrderTime::SpecifiedDay,
            expiration: 1_700_000_000,
            comment: "note".to_string(),
            position: 99,
            position_by: 100,
        };
        let body = trade_request(&request).unwrap();
        assert_eq!(body.len(), REQUEST_BYTES);

        let mut c = Cursor::new(&body);
        assert_eq!(c.i32("action").unwrap(), TradeAction::Pending.code());
        assert_eq!(c.u64("magic").unwrap(), 777);
        assert_eq!(c.u64("order").unwrap(), 88);
        assert_eq!(c.fixed_string(SLOT_SYMBOL, "symbol").unwrap(), "EURUSD");
        assert_eq!(c.f64("volume").unwrap(), 0.1);
        assert_eq!(c.f64("price").unwrap(), 1.2345);
        assert_eq!(c.f64("stoplimit").unwrap(), 1.2000);
        assert_eq!(c.f64("sl").unwrap(), 1.1000);
        assert_eq!(c.f64("tp").unwrap(), 1.3000);
        assert_eq!(c.u64("deviation").unwrap(), 20);
        assert_eq!(c.i32("type").unwrap(), OrderType::SellStopLimit.code());
        assert_eq!(c.i32("filling").unwrap(), OrderFilling::Boc.code());
        assert_eq!(c.i32("time").unwrap(), OrderTime::SpecifiedDay.code());
        assert_eq!(c.i64("expiration").unwrap(), 1_700_000_000);
        assert_eq!(c.fixed_string(SLOT_COMMENT, "comment").unwrap(), "note");
        assert_eq!(c.u64("position").unwrap(), 99);
        assert_eq!(c.u64("position_by").unwrap(), 100);
        c.expect_consumed("trade request").unwrap();
    }

    /// Enumerations travel as `i32` because that is what MQL5 declares them,
    /// but they used to be written as `u32`. For every value this crate can
    /// send, the two produce the same bytes, which is why the change was
    /// invisible to the terminal. The test says so rather than leaving it to
    /// be reasoned about.
    #[test]
    fn the_signed_and_unsigned_encodings_agree_for_every_value_we_send() {
        let sendable = [
            TradeAction::Deal.code(),
            TradeAction::Pending.code(),
            TradeAction::Sltp.code(),
            TradeAction::Modify.code(),
            TradeAction::Remove.code(),
            TradeAction::CloseBy.code(),
            OrderType::Buy.code(),
            OrderType::SellStopLimit.code(),
            OrderType::CloseBy.code(),
            OrderFilling::Fok.code(),
            OrderFilling::Boc.code(),
            OrderTime::Gtc.code(),
            OrderTime::SpecifiedDay.code(),
        ];
        for value in sendable {
            assert!(value >= 0, "{value} would differ between the two widths");
            assert_eq!(
                value.to_le_bytes(),
                (value as u32).to_le_bytes(),
                "value {value} encodes differently as i32 and u32"
            );
        }
    }

    /// A comment too long for its slot must not push the fields after it.
    #[test]
    fn an_over_long_comment_does_not_move_the_tail() {
        let request =
            TradeRequest::market("EURUSD", OrderType::Buy, 0.1, 1.0).comment(&"x".repeat(500));
        let body = trade_request(&request).unwrap();
        assert_eq!(body.len(), REQUEST_BYTES);
    }

    #[test]
    fn results_decode_and_short_ones_are_refused() {
        assert!(trade_result(&[0; 100]).is_err());
        assert!(check_result(&[0; 100]).is_err());
        let mut buf = vec![0u8; TRADE_RESULT_BYTES];
        buf[0..4].copy_from_slice(&10009u32.to_le_bytes());
        buf[4..12].copy_from_slice(&42u64.to_le_bytes());
        buf[252..256].copy_from_slice(&7u32.to_le_bytes());
        let r = trade_result(&buf).unwrap();
        assert!(r.is_success());
        assert_eq!((r.deal, r.request_id), (42, 7));
    }
}
