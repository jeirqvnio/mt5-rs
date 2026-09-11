//! `account_info`, `terminal_info`, `version`.

use crate::codec::fields::single;
use crate::error::Result;
use crate::types::{AccountInfo, TerminalInfo, TerminalVersion};
use crate::wire::{utf16, Cursor};

/// `terminal_info` is not packed: its strings sit at fixed offsets.
const TERMINAL_STRINGS: [usize; 6] = [41, 561, 1081, 1601, 2121, 2641];

pub fn account_info(buf: &[u8]) -> Result<AccountInfo> {
    single(buf, "account_info", |c| {
        fields!(c, "account", {
            login: i64, trade_mode: i32, leverage: i32, limit_orders: i32, margin_so_mode: i32,
            trade_allowed: bool, trade_expert: bool, margin_mode: i32, currency_digits: i32,
            fifo_close: bool, balance: f64, credit: f64, profit: f64, equity: f64, margin: f64,
            margin_free: f64, margin_level: f64, margin_so_call: f64, margin_so_so: f64,
            margin_initial: f64, margin_maintenance: f64, assets: f64, liabilities: f64,
            commission_blocked: f64,
            // 704-byte string block, measured: name 256, server 128, currency 64, company 256.
            name: fixed_string(256), server: fixed_string(128), currency: fixed_string(64),
            company: fixed_string(256),
        });
        Ok(AccountInfo {
            login,
            trade_mode,
            leverage,
            limit_orders,
            margin_so_mode,
            trade_allowed,
            trade_expert,
            margin_mode,
            currency_digits,
            fifo_close,
            balance,
            credit,
            profit,
            equity,
            margin,
            margin_free,
            margin_level,
            margin_so_call,
            margin_so_so,
            margin_initial,
            margin_maintenance,
            assets,
            liabilities,
            commission_blocked,
            name,
            server,
            currency,
            company,
        })
    })
}

pub fn terminal_info(buf: &[u8]) -> Result<TerminalInfo> {
    let mut c = Cursor::new(buf);
    fields!(c, "terminal", {
        build: u16, community_account: bool, community_connection: bool,
        notifications_enabled: bool, mqid: bool, connected: bool, dlls_allowed: bool,
        trade_allowed: bool, tradeapi_disabled: bool, email_enabled: bool, ftp_enabled: bool,
        maxbars: u32, _pad: skip(1), codepage: u16, _reserved: skip(2), ping_last: u32,
        community_balance: f64, retransmission: f64,
    });
    // The numeric block runs straight into the first string.
    c.expect_at(TERMINAL_STRINGS[0], "terminal_info numeric block")?;
    let at = |i: usize| {
        buf.get(TERMINAL_STRINGS[i]..)
            .map(utf16)
            .unwrap_or_default()
    };
    Ok(TerminalInfo {
        build,
        community_account,
        community_connection,
        notifications_enabled,
        mqid,
        connected,
        dlls_allowed,
        trade_allowed,
        tradeapi_disabled,
        email_enabled,
        ftp_enabled,
        maxbars,
        codepage,
        ping_last,
        community_balance,
        retransmission,
        company: at(0),
        name: at(1),
        language: at(2),
        path: at(3),
        data_path: at(4),
        commondata_path: at(5),
    })
}

/// `500 | 6090 | "31 Jul 2026"`.
pub fn version(buf: &[u8]) -> Result<TerminalVersion> {
    single(buf, "version", |c| {
        fields!(c, "version", { api: u32, build: u32, released: string });
        Ok(TerminalVersion {
            api,
            build,
            released,
        })
    })
}
