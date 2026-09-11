//! `symbol_info` / `symbols_get`: ~90 fields, of which the model keeps the
//! ones that matter. The rest are read and dropped to keep the offsets.

use crate::codec::fields::{batch, single};
use crate::error::Result;
use crate::types::SymbolInfo;
use crate::wire::Cursor;

const SYMBOL_MIN: usize = 2993;

fn symbol(c: &mut Cursor) -> Result<SymbolInfo> {
    fields!(c, "symbol", {
        custom: bool, _chart_mode: skip(4), select: bool, visible: bool,
        _session_counters: skip(24), _volumes: skip(24), time: i64, digits: i32, spread: i32,
        spread_float: bool, ticks_bookdepth: i32, trade_calc_mode: i32, trade_mode: i32,
        start_time: i64, expiration_time: i64, trade_stops_level: i32, trade_freeze_level: i32,
        trade_exemode: i32, swap_mode: i32, swap_rollover3days: i32, _margin_hedged_use_leg: skip(1),
        expiration_mode: i32, filling_mode: i32, order_mode: i32, order_gtc_mode: i32,
        _option_mode_and_right: skip(8), bid: f64, _bid_high_low: skip(16), ask: f64,
        _ask_high_low: skip(16), last: f64, _last_high_low: skip(16), volume_real: f64,
        _volume_high_low_real: skip(16), _option_strike: skip(8), point: f64,
        trade_tick_value: f64, trade_tick_value_profit: f64, trade_tick_value_loss: f64,
        trade_tick_size: f64, trade_contract_size: f64, _accrued_face_liquidity: skip(24),
        volume_min: f64, volume_max: f64, volume_step: f64, volume_limit: f64, swap_long: f64,
        swap_short: f64, margin_initial: f64, margin_maintenance: f64, _session_stats: skip(40),
        session_open: f64, session_close: f64, _session_aw_and_limits: skip(32),
        margin_hedged: f64, price_change: f64, price_volatility: f64, _price_theoretical: skip(8),
        _greeks: skip(48), _price_sensitivity: skip(8),
        // String slots, 2432 bytes in total.
        _basis: skip(64), _category: skip(128), currency_base: fixed_string(32),
        currency_profit: fixed_string(32), currency_margin: fixed_string(32), _bank: skip(512),
        description: fixed_string(64), exchange: fixed_string(64), _formula: skip(1024),
        isin: fixed_string(32), _page: skip(128), path: fixed_string(256), name: fixed_string(64),
    });
    Ok(SymbolInfo {
        name,
        custom,
        select,
        visible,
        time,
        digits,
        spread,
        spread_float,
        ticks_bookdepth,
        trade_calc_mode,
        trade_mode,
        start_time,
        expiration_time,
        trade_stops_level,
        trade_freeze_level,
        trade_exemode,
        swap_mode,
        swap_rollover3days,
        expiration_mode,
        filling_mode,
        order_mode,
        order_gtc_mode,
        bid,
        ask,
        last,
        volume_real,
        point,
        trade_tick_value,
        trade_tick_value_profit,
        trade_tick_value_loss,
        trade_tick_size,
        trade_contract_size,
        volume_min,
        volume_max,
        volume_step,
        volume_limit,
        swap_long,
        swap_short,
        margin_initial,
        margin_maintenance,
        session_open,
        session_close,
        margin_hedged,
        price_change,
        price_volatility,
        currency_base,
        currency_profit,
        currency_margin,
        description,
        exchange,
        isin,
        path,
    })
}

pub fn symbol_one(buf: &[u8]) -> Result<SymbolInfo> {
    single(buf, "symbol", symbol)
}

pub fn symbols(buf: &[u8]) -> Result<Vec<SymbolInfo>> {
    batch(buf, SYMBOL_MIN, "symbols", symbol)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// The record is exactly SYMBOL_MIN bytes: the table above must add up.
    #[test]
    fn the_symbol_table_sums_to_the_record_size() {
        let zeros = vec![0u8; SYMBOL_MIN];
        assert!(symbol_one(&zeros).is_ok());
        assert!(symbol_one(&zeros[..SYMBOL_MIN - 1]).is_err());
    }
}
