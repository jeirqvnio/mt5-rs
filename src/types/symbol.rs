//! Contract terms, and the arithmetic that keeps an order on the contract's
//! own grid.

use crate::types::{filling_mask, order_filling};

/// How close to an exact grid multiple counts as on it.
const GRID_TOLERANCE: f64 = 1e-9;

/// The terminal sends ~90 fields; these are the ones that bear on quoting,
/// sizing and order construction.
#[derive(Debug, Clone, Default)]
pub struct SymbolInfo {
    pub name: String,
    pub custom: bool,
    pub select: bool,
    pub visible: bool,
    pub time: i64,
    pub digits: i32,
    pub spread: i32,
    pub spread_float: bool,
    pub ticks_bookdepth: i32,
    pub trade_calc_mode: i32,
    pub trade_mode: i32,
    pub start_time: i64,
    pub expiration_time: i64,
    /// Minimum distance of SL/TP from price, in points.
    pub trade_stops_level: i32,
    /// Band around price inside which an order cannot be modified, in points.
    pub trade_freeze_level: i32,
    pub trade_exemode: i32,
    pub swap_mode: i32,
    pub swap_rollover3days: i32,
    pub expiration_mode: i32,
    /// Bitmask of [`filling_mask`].
    pub filling_mode: i32,
    pub order_mode: i32,
    pub order_gtc_mode: i32,
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub volume_real: f64,
    pub point: f64,
    pub trade_tick_value: f64,
    pub trade_tick_value_profit: f64,
    pub trade_tick_value_loss: f64,
    pub trade_tick_size: f64,
    pub trade_contract_size: f64,
    pub volume_min: f64,
    pub volume_max: f64,
    pub volume_step: f64,
    pub volume_limit: f64,
    pub swap_long: f64,
    pub swap_short: f64,
    pub margin_initial: f64,
    pub margin_maintenance: f64,
    pub session_open: f64,
    pub session_close: f64,
    pub margin_hedged: f64,
    pub price_change: f64,
    pub price_volatility: f64,
    pub currency_base: String,
    pub currency_profit: String,
    pub currency_margin: String,
    pub description: String,
    pub exchange: String,
    pub isin: String,
    pub path: String,
}

impl SymbolInfo {
    /// Whether the contract advertises this filling mode. `RETURN` never
    /// appears in the mask, so it always reads false here even where the
    /// broker accepts it for pending orders.
    pub fn supports_filling(&self, mode: u32) -> bool {
        let bit = match mode {
            order_filling::FOK => filling_mask::FOK,
            order_filling::IOC => filling_mask::IOC,
            order_filling::BOC => filling_mask::BOC,
            // RETURN is never advertised in the mask.
            _ => return false,
        };
        self.filling_mode & bit != 0
    }

    /// The filling mode to send, or `None` when the contract advertises none.
    ///
    /// Not optional in practice: the default is FOK, and a broker that does
    /// not offer it answers `INVALID_FILL` before looking at anything else.
    pub fn preferred_filling(&self) -> Option<u32> {
        [order_filling::FOK, order_filling::IOC, order_filling::BOC]
            .into_iter()
            .find(|&m| self.supports_filling(m))
    }

    /// Cap and snap a volume onto the lot grid, rounding down. `None` when it
    /// cannot be represented: no step, or below the minimum lot.
    pub fn normalize_volume(&self, volume: f64) -> Option<f64> {
        if !volume.is_finite() || self.volume_step <= 0.0 {
            return None;
        }
        // 0.3 / 0.1 is 2.9999999999999996; floor only between genuine steps.
        let raw = volume.min(self.volume_max) / self.volume_step;
        let nearest = raw.round();
        let steps = if (raw - nearest).abs() < GRID_TOLERANCE {
            nearest
        } else {
            raw.floor()
        };
        let snapped = round_to(steps * self.volume_step, decimals_of(self.volume_step));
        (snapped + f64::EPSILON >= self.volume_min).then_some(snapped)
    }

    /// Round a price onto the tick grid and the contract's digits.
    pub fn normalize_price(&self, price: f64) -> f64 {
        let digits = self.digits.max(0) as u32;
        if self.trade_tick_size > 0.0 {
            round_to(
                (price / self.trade_tick_size).round() * self.trade_tick_size,
                digits,
            )
        } else {
            round_to(price, digits)
        }
    }

    /// Minimum SL/TP distance in price units.
    pub fn min_stop_distance(&self) -> f64 {
        f64::from(self.trade_stops_level.max(0)) * self.point
    }
}

fn decimals_of(step: f64) -> u32 {
    let mut d = 0;
    let mut v = step;
    while d < 8 && (v - v.round()).abs() > 1e-9 {
        v *= 10.0;
        d += 1;
    }
    d
}

fn round_to(v: f64, decimals: u32) -> f64 {
    let f = 10f64.powi(decimals as i32);
    (v * f).round() / f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbol() -> SymbolInfo {
        SymbolInfo {
            volume_min: 0.01,
            volume_max: 100.0,
            volume_step: 0.01,
            digits: 5,
            trade_tick_size: 0.00001,
            point: 0.00001,
            trade_stops_level: 10,
            filling_mode: filling_mask::IOC,
            ..Default::default()
        }
    }

    #[test]
    fn volume_snaps_down_and_stays_exact() {
        let s = symbol();
        assert_eq!(s.normalize_volume(0.123), Some(0.12));
        assert_eq!(s.normalize_volume(500.0), Some(100.0));
        assert_eq!(s.normalize_volume(0.004), None);
        assert_eq!(s.normalize_volume(f64::NAN), None);
        let coarse = SymbolInfo {
            volume_step: 0.1,
            volume_min: 0.1,
            ..symbol()
        };
        assert_eq!(
            coarse.normalize_volume(0.3),
            Some(0.3),
            "0.3/0.1 floors to 2 in binary"
        );
        assert_eq!(s.normalize_volume(1.0), Some(1.0), "1.0/0.01 is 99.999…");
    }

    #[test]
    fn price_and_filling_follow_the_contract() {
        let s = symbol();
        assert_eq!(s.normalize_price(1.234567), 1.23457);
        assert_eq!(s.min_stop_distance(), 0.0001);
        assert_eq!(s.preferred_filling(), Some(order_filling::IOC));
        assert_eq!(SymbolInfo::default().preferred_filling(), None);
    }
}
