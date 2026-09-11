//! Market Watch: the contracts and their quotes.

use crate::client::{commands, Mt5};
use crate::codec;
use crate::error::{Error, Result};
use crate::types::{SymbolInfo, Tick};
use crate::wire::Writer;

impl Mt5 {
    /// How many symbols the terminal knows, without decoding any of them.
    pub async fn symbols_total(&self) -> Result<u32> {
        self.call_u32("symbols_total", commands::SYMBOLS_TOTAL, &[])
            .await
    }

    /// All symbols, or those matching a Market Watch mask (`*USD*`), which the
    /// terminal filters itself.
    pub async fn symbols(&self, group: Option<&str>) -> Result<Vec<SymbolInfo>> {
        let raw = match group {
            Some(mask) => {
                let body = Writer::new().string(mask).into_bytes();
                self.call("symbols", commands::SYMBOLS_GET_GROUP, &body)
                    .await?
            }
            None => self.call("symbols", commands::SYMBOLS_GET, &[]).await?,
        };
        codec::symbols(&raw)
    }

    /// A contract's terms: digits, point, lot grid, tick value, the minimum
    /// stop distance and which filling modes it accepts.
    ///
    /// # Errors
    /// [`crate::Error::Refused`] when the terminal does not know the symbol.
    /// A symbol outside Market Watch has to be added with
    /// [`Mt5::symbol_select`] first.
    pub async fn symbol_info(&self, symbol: &str) -> Result<SymbolInfo> {
        let body = Writer::new().string(symbol).into_bytes();
        self.call_opt(
            "symbol_info",
            commands::SYMBOL_INFO,
            &body,
            codec::symbol_one,
        )
        .await?
        .ok_or_else(|| Error::Refused(format!("symbol `{symbol}` is unknown to the terminal")))
    }

    /// The current quote: bid, ask, last, and the millisecond timestamp that
    /// [`Mt5::ticks_from`] takes as its mark.
    ///
    /// # Errors
    /// [`crate::Error::Refused`] when the symbol is not in Market Watch, so
    /// the terminal has no quote to give.
    pub async fn symbol_tick(&self, symbol: &str) -> Result<Tick> {
        let body = Writer::new().string(symbol).into_bytes();
        self.call_opt(
            "symbol_tick",
            commands::SYMBOL_INFO_TICK,
            &body,
            codec::tick_one,
        )
        .await?
        .ok_or_else(|| Error::Refused(format!("no tick for `{symbol}`; is it in Market Watch?")))
    }

    /// Add to or drop from Market Watch. Needed before quoting or trading a
    /// symbol the terminal was not already watching.
    pub async fn symbol_select(&self, symbol: &str, enable: bool) -> Result<bool> {
        let body = Writer::new().string(symbol).bool(enable).into_bytes();
        self.call_ack("symbol_select", commands::SYMBOL_SELECT, &body)
            .await
    }
}
