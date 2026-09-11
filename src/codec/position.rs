//! Positions, orders and deals: one record shape each, read as a batch or
//! as a single by-ticket lookup.

use crate::codec::fields::{batch, single};
use crate::codec::{SLOT_COMMENT, SLOT_EXTERNAL_ID, SLOT_SYMBOL};
use crate::error::Result;
use crate::types::{Deal, Order, Position};
use crate::wire::Cursor;

const POSITION_MIN: usize = 320;
const ORDER_MIN: usize = 340;
const DEAL_MIN: usize = 300;

fn position(c: &mut Cursor) -> Result<Position> {
    fields!(c, "position", {
        ticket: u64, time: i64, time_msc: i64, time_update: i64, time_update_msc: i64,
        kind: enum_i32, magic: u64, identifier: u64, reason: i32, volume: f64, price_open: f64, sl: f64, tp: f64,
        price_current: f64, swap: f64,
        // Eight bytes the Python API does not expose; reads zero intraday.
        _unknown: skip(8),
        profit: f64, symbol: fixed_string(SLOT_SYMBOL), comment: fixed_string(SLOT_COMMENT),
        external_id: fixed_string(SLOT_EXTERNAL_ID),
    });
    Ok(Position {
        ticket,
        time,
        time_msc,
        time_update,
        time_update_msc,
        kind,
        magic,
        identifier,
        reason,
        volume,
        price_open,
        sl,
        tp,
        price_current,
        swap,
        profit,
        symbol,
        comment,
        external_id,
    })
}

pub fn positions(buf: &[u8]) -> Result<Vec<Position>> {
    batch(buf, POSITION_MIN, "positions", position)
}

pub fn position_one(buf: &[u8]) -> Result<Position> {
    single(buf, "position", position)
}

fn order(c: &mut Cursor) -> Result<Order> {
    fields!(c, "order", {
        ticket: u64, time_setup: i64, time_setup_msc: i64, time_done: i64, time_done_msc: i64,
        time_expiration: i64, kind: enum_i32, type_time: enum_i32, type_filling: enum_i32,
        state: enum_i32, magic: u64,
        position_id: u64, position_by_id: u64, reason: i32, volume_initial: f64,
        volume_current: f64, price_open: f64, price_current: f64, sl: f64, tp: f64,
        price_stoplimit: f64, symbol: fixed_string(SLOT_SYMBOL), comment: fixed_string(SLOT_COMMENT),
        external_id: fixed_string(SLOT_EXTERNAL_ID),
    });
    Ok(Order {
        ticket,
        time_setup,
        time_setup_msc,
        time_done,
        time_done_msc,
        time_expiration,
        kind,
        type_time,
        type_filling,
        state,
        magic,
        position_id,
        position_by_id,
        reason,
        volume_initial,
        volume_current,
        price_open,
        price_current,
        sl,
        tp,
        price_stoplimit,
        symbol,
        comment,
        external_id,
    })
}

pub fn orders(buf: &[u8]) -> Result<Vec<Order>> {
    batch(buf, ORDER_MIN, "orders", order)
}

pub fn order_one(buf: &[u8]) -> Result<Order> {
    single(buf, "order", order)
}

fn deal(c: &mut Cursor) -> Result<Deal> {
    fields!(c, "deal", {
        ticket: u64, order: u64, time: i64, time_msc: i64, kind: enum_i32, entry: enum_i32,
        magic: u64,
        position_id: u64, reason: i32, volume: f64, price: f64, commission: f64, swap: f64,
        profit: f64, fee: f64, symbol: fixed_string(SLOT_SYMBOL), comment: fixed_string(SLOT_COMMENT),
        external_id: fixed_string(SLOT_EXTERNAL_ID),
    });
    Ok(Deal {
        ticket,
        order,
        time,
        time_msc,
        kind,
        entry,
        magic,
        position_id,
        reason,
        volume,
        price,
        commission,
        swap,
        profit,
        fee,
        symbol,
        comment,
        external_id,
    })
}

pub fn deals(buf: &[u8]) -> Result<Vec<Deal>> {
    batch(buf, DEAL_MIN, "deals", deal)
}

pub fn deal_one(buf: &[u8]) -> Result<Deal> {
    single(buf, "deal", deal)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::wire::Writer;

    #[test]
    fn an_empty_count_is_an_empty_set_and_a_short_record_names_its_field() {
        let empty = Writer::new().u32(0).into_bytes();
        assert!(positions(&empty).unwrap().is_empty());
        assert!(orders(&empty).unwrap().is_empty());
        assert!(deals(&empty).unwrap().is_empty());
        let err = position_one(&[0u8; 100]).unwrap_err().to_string();
        assert!(err.contains("position."), "{err}");
    }
}
