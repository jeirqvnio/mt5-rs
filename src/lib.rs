//! MetaTrader 5 from Rust, over the terminal's own IPC. No Python anywhere.
//!
//! ```no_run
//! # async fn run() -> mt5::Result<()> {
//! let mt5 = mt5::Mt5::relay("127.0.0.1:18813", "token").await?;   // or Mt5::terminal(path) on Windows
//! mt5.login(1_000_000, "password", "Broker-Demo").await?;
//! let tick = mt5.symbol_tick("EURUSD").await?;
//! let symbol = mt5.symbol_info("EURUSD").await?;
//! let mut request = mt5::TradeRequest::market("EURUSD", mt5::OrderType::Buy, 0.1, tick.ask)
//!     .deviation(20)
//!     .magic(42);
//! if let Some(mode) = symbol.preferred_filling() {
//!     request = request.filling(mode);
//! }
//! let result = mt5.order_send(&request).await?;
//! println!("{} {}", result.retcode, result.comment);
//! # Ok(())
//! # }
//! ```
//!
//! MetaQuotes does not document this protocol; the layouts were reconstructed
//! by observation against builds 5836 to 6182. Every read is bounds-checked and
//! must consume its record exactly, so a build that moves a field fails by
//! name instead of returning a plausible number. The trade request is
//! size-checked before it leaves, `order_check` exercises it without reaching
//! a book, and `order_send` is never retried.

// Everything a caller needs is re-exported below, so `mt5::Mt5` and
// `mt5::TradeRequest` are the whole import list for ordinary use.
pub mod client;
/// Record layouts: the decoders are internal, but their field tables are the
/// clearest statement of what the protocol looks like.
mod codec;
/// Errors, and what they mean for whether a call may be repeated.
pub mod error;
pub mod types;
pub mod wire;

pub use client::{Config, Credentials, Mt5};
pub use error::{Error, Result};
pub use types::*;
pub use wire::{pipe_name_for, Endpoint};

/// Keep the ticks newer than `mark` and move it past them.
///
/// The terminal answers `ticks_from` *from* the mark inclusive, so the newest
/// tick of one poll is the oldest of the next. The mark never moves backwards.
///
/// ```no_run
/// # async fn run(mt5: mt5::Mt5) -> mt5::Result<()> {
/// let mut mark = 0;   // zero: start from the newest
/// loop {
///     let batch = mt5.ticks_from("EURUSD", mark, 2_000, mt5::copy_ticks::ALL).await?;
///     for tick in mt5::advance_ticks(&mut mark, batch) {
///         println!("{} {}", tick.time_msc, tick.bid);
///     }
/// }
/// # }
/// ```
pub fn advance_ticks(mark: &mut i64, batch: Vec<Tick>) -> Vec<Tick> {
    let fresh: Vec<Tick> = batch.into_iter().filter(|t| t.time_msc > *mark).collect();
    if let Some(newest) = fresh.iter().map(|t| t.time_msc).max() {
        *mark = newest;
    }
    fresh
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(time_msc: i64) -> Tick {
        Tick {
            time_msc,
            ..Tick::default()
        }
    }

    #[test]
    fn advance_drops_the_overlap_and_never_rewinds() {
        let mut mark = 1_000;
        assert_eq!(
            advance_ticks(&mut mark, vec![at(1_000), at(1_100), at(1_200)]).len(),
            2
        );
        assert_eq!(mark, 1_200);
        assert!(advance_ticks(&mut mark, vec![at(100)]).is_empty());
        assert_eq!(mark, 1_200);
        let mut zero = 0;
        assert_eq!(advance_ticks(&mut zero, vec![at(5), at(6)]).len(), 2);
    }
}
