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
    use crate::types::{OrderType, TradeAction};

    #[test]
    fn the_request_is_232_bytes_with_fields_where_the_terminal_expects_them() {
        let r = TradeRequest::market("EURUSD", OrderType::Buy, 0.1, 1.2345)
            .magic(777)
            .deviation(20)
            .comment(&"x".repeat(500));
        let body = trade_request(&r).unwrap();
        assert_eq!(body.len(), REQUEST_BYTES);
        let mut c = Cursor::new(&body);
        assert_eq!(c.i32("action").unwrap(), TradeAction::Deal.code());
        assert_eq!(c.u64("magic").unwrap(), 777);
        c.skip(8, "order").unwrap();
        assert_eq!(c.fixed_string(64, "symbol").unwrap(), "EURUSD");
        assert_eq!(c.f64("volume").unwrap(), 0.1);
        assert_eq!(c.f64("price").unwrap(), 1.2345);
        c.skip(24, "stoplimit sl tp").unwrap();
        assert_eq!(c.u64("deviation").unwrap(), 20);
        assert_eq!(c.i32("type").unwrap(), OrderType::Buy.code());
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
