//! Bars, ticks and the order book.

use crate::client::{commands, Mt5};
use crate::codec;
use crate::error::Result;
use crate::types::{BookEntry, Rate, Tick, Timeframe};
use crate::wire::Writer;

impl Mt5 {
    /// `count` bars ending `start_pos` bars back from the newest.
    pub async fn rates_from_pos(
        &self,
        symbol: &str,
        tf: Timeframe,
        start_pos: u32,
        count: u32,
    ) -> Result<Vec<Rate>> {
        let body = Writer::new()
            .string(symbol)
            .u32(tf.code())
            .u32(start_pos)
            .u32(count)
            .into_bytes();
        codec::rates(
            &self
                .call("rates_from_pos", commands::COPY_RATES_FROM_POS, &body)
                .await?,
        )
    }

    /// `count` bars starting at `from` (Unix seconds).
    pub async fn rates_from(
        &self,
        symbol: &str,
        tf: Timeframe,
        from: i64,
        count: u32,
    ) -> Result<Vec<Rate>> {
        let body = Writer::new()
            .string(symbol)
            .u32(tf.code())
            .i64(from)
            .u32(count)
            .into_bytes();
        codec::rates(
            &self
                .call("rates_from", commands::COPY_RATES_FROM, &body)
                .await?,
        )
    }

    /// Bars between two times (Unix seconds).
    pub async fn rates_range(
        &self,
        symbol: &str,
        tf: Timeframe,
        from: i64,
        to: i64,
    ) -> Result<Vec<Rate>> {
        let body = Writer::new()
            .string(symbol)
            .u32(tf.code())
            .i64(from)
            .i64(to)
            .into_bytes();
        codec::rates(
            &self
                .call("rates_range", commands::COPY_RATES_RANGE, &body)
                .await?,
        )
    }

    /// Up to `count` ticks from `from_msc` inclusive. **Milliseconds** — the
    /// `time_msc` of a tick. Seconds are accepted silently and answer from
    /// hours in the past; zero means the newest. See [`crate::advance_ticks`].
    pub async fn ticks_from(
        &self,
        symbol: &str,
        from_msc: i64,
        count: u32,
        flags: u32,
    ) -> Result<Vec<Tick>> {
        let body = Writer::new()
            .string(symbol)
            .i64(from_msc)
            .u32(count)
            .u32(flags)
            .into_bytes();
        codec::ticks(
            &self
                .call("ticks_from", commands::COPY_TICKS_FROM, &body)
                .await?,
        )
    }

    /// Ticks between two marks, both in milliseconds.
    pub async fn ticks_range(
        &self,
        symbol: &str,
        from_msc: i64,
        to_msc: i64,
        flags: u32,
    ) -> Result<Vec<Tick>> {
        let body = Writer::new()
            .string(symbol)
            .i64(from_msc)
            .i64(to_msc)
            .u32(flags)
            .into_bytes();
        codec::ticks(
            &self
                .call("ticks_range", commands::COPY_TICKS_RANGE, &body)
                .await?,
        )
    }

    /// Subscribe to a symbol's level 2 book. Required before
    /// [`Mt5::book_get`] returns anything, and only some brokers publish one.
    pub async fn book_add(&self, symbol: &str) -> Result<bool> {
        let body = Writer::new().string(symbol).into_bytes();
        self.call_ack("book_add", commands::MARKET_BOOK_ADD, &body)
            .await
    }

    /// The current level 2 book. Empty until [`Mt5::book_add`] has been
    /// called for the symbol, and outside trading hours.
    pub async fn book_get(&self, symbol: &str) -> Result<Vec<BookEntry>> {
        let body = Writer::new().string(symbol).into_bytes();
        codec::book(
            &self
                .call("book_get", commands::MARKET_BOOK_GET, &body)
                .await?,
        )
    }

    /// Drop the level 2 subscription. The terminal keeps publishing until
    /// this is called, so it costs bandwidth to forget.
    pub async fn book_release(&self, symbol: &str) -> Result<bool> {
        let body = Writer::new().string(symbol).into_bytes();
        self.call_ack("book_release", commands::MARKET_BOOK_RELEASE, &body)
            .await
    }
}
