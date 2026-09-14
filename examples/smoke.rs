//! Read-only: does everything answer? `MT5_RELAY_ADDR` + `MT5_BRIDGE_TOKEN`,
//! or `MT5_PATH` on Windows. `MT5_SYMBOL` defaults to EURUSD.

use mt5::{copy_ticks, Config, Mt5, Timeframe};

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let config = match std::env::var("MT5_PATH") {
        Ok(path) => Config::terminal(&path),
        Err(_) => Config::relay(
            &std::env::var("MT5_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18813".into()),
            &std::env::var("MT5_BRIDGE_TOKEN").unwrap_or_default(),
        ),
    };
    let symbol = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "EURUSD".into());
    let mt5 = Mt5::new(config);
    mt5.connect().await?;

    let version = mt5.version().await?;
    let terminal = mt5.terminal_info().await?;
    println!(
        "build {} ({}), connected {}, algo trading {}",
        version.build, version.released, terminal.connected, terminal.trade_allowed
    );
    let account = mt5.account_info().await?;
    if account.login == 0 {
        println!("no account logged in; set one with Config::account or Mt5::login");
        return Ok(());
    }
    println!(
        "account {} {} {} {}",
        account.login, account.server, account.balance, account.currency
    );
    println!("symbols {}", mt5.symbols_total().await?);

    mt5.symbol_select(&symbol, true).await?;
    let info = mt5.symbol_info(&symbol).await?;
    let tick = mt5.symbol_tick(&symbol).await?;
    println!(
        "{symbol}: bid {} ask {} digits {} lot {}..{} step {} filling {:?}",
        tick.bid,
        tick.ask,
        info.digits,
        info.volume_min,
        info.volume_max,
        info.volume_step,
        info.preferred_filling()
    );
    let bars = mt5.rates_from_pos(&symbol, Timeframe::M1, 0, 3).await?;
    for b in &bars {
        println!(
            "  bar {} o {} h {} l {} c {}",
            b.time, b.open, b.high, b.low, b.close
        );
    }
    let ticks = mt5.ticks_from(&symbol, 0, 5, copy_ticks::ALL).await?;
    println!(
        "  {} ticks, newest {:?}",
        ticks.len(),
        ticks.last().map(|t| t.time_msc)
    );

    println!(
        "positions {}, orders {}",
        mt5.positions_total().await?,
        mt5.orders_total().await?
    );
    for p in mt5.positions(None).await? {
        println!(
            "  #{} {} {} {} @ {} pnl {}",
            p.ticket, p.symbol, p.kind, p.volume, p.price_open, p.profit
        );
    }
    let now = tick.time;
    println!(
        "deals last 24h: {}",
        mt5.history_deals(now - 86_400, now + 3_600).await?.len()
    );
    Ok(())
}
