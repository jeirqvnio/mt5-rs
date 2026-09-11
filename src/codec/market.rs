//! Ticks, bars and the order book.

use crate::codec::fields::{batch, single};
use crate::error::Result;
use crate::types::{BookEntry, Rate, Tick};
use crate::wire::Cursor;

const RATE_MIN: usize = 60;
const TICK_MIN: usize = 60;
const BOOK_MIN: usize = 32;

fn tick(c: &mut Cursor) -> Result<Tick> {
    fields!(c, "tick", {
        time: i64, bid: f64, ask: f64, last: f64, volume: u64, time_msc: i64, flags: u32,
        volume_real: f64,
    });
    Ok(Tick {
        time,
        bid,
        ask,
        last,
        volume,
        time_msc,
        flags,
        volume_real,
    })
}

pub fn tick_one(buf: &[u8]) -> Result<Tick> {
    single(buf, "tick", tick)
}

pub fn ticks(buf: &[u8]) -> Result<Vec<Tick>> {
    batch(buf, TICK_MIN, "ticks", tick)
}

pub fn rates(buf: &[u8]) -> Result<Vec<Rate>> {
    batch(buf, RATE_MIN, "rates", |c| {
        fields!(c, "rate", {
            time: i64, open: f64, high: f64, low: f64, close: f64, tick_volume: i64, spread: i32,
            real_volume: i64,
        });
        Ok(Rate {
            time,
            open,
            high,
            low,
            close,
            tick_volume,
            spread,
            real_volume,
        })
    })
}

pub fn book(buf: &[u8]) -> Result<Vec<BookEntry>> {
    batch(buf, BOOK_MIN, "book", |c| {
        fields!(c, "book", { kind: i64, price: f64, volume: i64, volume_real: f64 });
        Ok(BookEntry {
            kind,
            price,
            volume,
            volume_real,
        })
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::wire::Writer;

    #[test]
    fn a_truncated_or_padded_batch_is_rejected() {
        let mut short = Writer::new().u32(2).into_bytes();
        short.extend(std::iter::repeat(0u8).take(RATE_MIN));
        assert!(rates(&short).is_err());
        let mut long = Writer::new().u32(1).into_bytes();
        long.extend(std::iter::repeat(0u8).take(RATE_MIN + 8));
        assert!(rates(&long).unwrap_err().to_string().contains("layout"));
        assert!(rates(&[]).unwrap().is_empty());
    }

    #[test]
    fn a_rate_decodes_at_its_offsets() {
        let body = Writer::new()
            .u32(1)
            .i64(1_700_000_000)
            .f64(1.1)
            .f64(1.2)
            .f64(1.0)
            .f64(1.15)
            .i64(42)
            .u32(3)
            .i64(7)
            .into_bytes();
        let got = rates(&body).unwrap();
        assert_eq!(
            (got[0].time, got[0].close, got[0].spread, got[0].real_volume),
            (1_700_000_000, 1.15, 3, 7)
        );
    }
}
