//! DEMO ONLY. Open a market position at the smallest lot the contract
//! allows, read it back, then close it at market and account for it from the
//! server's own deal history.
//!
//! This is the one path a pending order cannot exercise: a fill. It refuses
//! to run on a real account, and the position exists for about a second — but
//! it is a real order, and it costs the spread. Same environment as `smoke`.

use std::time::Duration;

use mt5::{order_type, retcode, AccountInfo, Config, Mt5, SymbolInfo, TradeRequest};

/// `ENUM_ACCOUNT_TRADE_MODE`: 0 demo, 1 contest, 2 real.
const ACCOUNT_REAL: i32 = 2;

/// `SYMBOL_TRADE_MODE_FULL`: the broker accepts orders, not just quotes.
const TRADE_MODE_FULL: i32 = 4;

/// Stamped on both legs so the deals can be told from anything else on the
/// account.
const MAGIC: u64 = 20_260_911;

/// How long to let the server publish the closing deal into history.
const SETTLE: Duration = Duration::from_millis(1500);

/// How far behind the newest quote on the account a symbol's own quote may
/// sit and still count as trading, in seconds.
const STALE_QUOTE: i64 = 120;

/// How far below the entry the test stop goes, as a fraction of the price.
const STOP_FRACTION: f64 = 0.01;

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let config = Config::relay(
        &std::env::var("MT5_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18813".into()),
        &std::env::var("MT5_BRIDGE_TOKEN").unwrap_or_default(),
    );
    let mt5 = Mt5::connect(config).await?;

    let account = mt5.account_info().await?;
    let terminal = mt5.terminal_info().await?;
    println!(
        "account {} on {} — {} {}, trade_mode {}",
        account.login, account.server, account.balance, account.currency, account.trade_mode
    );
    if account.trade_mode == ACCOUNT_REAL {
        println!("REAL account: refusing to send a market order from an example.");
        return Ok(());
    }
    if !terminal.trade_allowed || !account.trade_allowed {
        println!(
            "trading is off (terminal {}, account {}); switch AutoTrading on",
            terminal.trade_allowed, account.trade_allowed
        );
        return Ok(());
    }

    let Some(symbol) = cheapest_round_trip(&mt5, &account).await? else {
        println!("no tradeable symbol with a live quote — is the market open?");
        return Ok(());
    };
    let tick = mt5.symbol_tick(&symbol.name).await?;
    let lot = symbol.volume_min;
    println!(
        "{}: bid {} ask {} spread {} | lot {} | filling {:?}",
        symbol.name,
        tick.bid,
        tick.ask,
        symbol.normalize_price(tick.ask - tick.bid),
        lot,
        symbol.preferred_filling()
    );

    // Ask first. A rejection here costs nothing and names the reason.
    let entry = market(&symbol, order_type::BUY, lot, tick.ask);
    let check = mt5.order_check(&entry).await?;
    println!(
        "order_check  {} — margin {} of {} free",
        retcode::name(check.retcode),
        check.margin,
        check.margin_free
    );
    if !check.is_ok() {
        println!("not sending: {}", check.comment);
        return Ok(());
    }

    let opened = mt5.order_send(&entry).await?;
    println!(
        "order_send   {} — deal {} order {} volume {} at {}",
        retcode::name(opened.retcode),
        opened.deal,
        opened.order,
        opened.volume,
        opened.price
    );
    if !opened.is_success() {
        println!("nothing opened: {}", opened.comment);
        return Ok(());
    }

    // The position carries the ticket of the order that opened it.
    let position = mt5
        .position(opened.order)
        .await?
        .ok_or_else(|| mt5::Error::Refused(format!("position {} not found", opened.order)))?;
    println!(
        "position     #{} {} {} at {} — magic {} profit {}",
        position.ticket,
        position.symbol,
        position.volume,
        position.price_open,
        position.magic,
        position.profit
    );
    assert_eq!(position.magic, MAGIC, "magic must survive the round trip");
    assert_eq!(position.volume, lot, "volume must be what was asked for");

    let account_open = mt5.account_info().await?;
    println!(
        "margin taken {} — equity {} vs balance {}",
        account_open.margin, account_open.equity, account_open.balance
    );

    // A stop one per cent below the entry: far enough that a second of
    // market cannot reach it, and well clear of the broker's own minimum
    // distance. Setting it must not disturb the target, which this position
    // does not have.
    let away = (position.price_open * STOP_FRACTION).max(symbol.min_stop_distance() * 2.0);
    let stop = symbol.normalize_price(position.price_open - away);
    let protected = mt5
        .order_send(&TradeRequest::protect(&position).sl(stop))
        .await?;
    println!(
        "protect      {} — stop asked {}",
        retcode::name(protected.retcode),
        stop
    );
    if protected.is_success() {
        let now = mt5
            .position(position.ticket)
            .await?
            .ok_or_else(|| mt5::Error::Refused("position vanished".into()))?;
        println!("  server says  sl {} tp {}", now.sl, now.tp);
    }

    // Close it: a deal the other way, against this position's ticket.
    let tick = mt5.symbol_tick(&symbol.name).await?;
    let mut exit = TradeRequest::close(&position, tick.bid).magic(MAGIC);
    if let Some(mode) = symbol.preferred_filling() {
        exit = exit.filling(mode);
    }
    let closed = mt5.order_send(&exit).await?;
    println!(
        "close        {} — deal {} volume {} at {}",
        retcode::name(closed.retcode),
        closed.deal,
        closed.volume,
        closed.price
    );

    tokio::time::sleep(SETTLE).await;
    println!("positions now {}", mt5.positions_total().await?);

    // What it actually cost, from the server's own record rather than ours.
    let deals = mt5.history_deals_of_position(position.ticket).await?;
    let mut realised = 0.0;
    for deal in &deals {
        println!(
            "  deal {} entry {} volume {} at {} profit {} commission {} swap {}",
            deal.ticket,
            deal.entry,
            deal.volume,
            deal.price,
            deal.profit,
            deal.commission,
            deal.swap
        );
        realised += deal.profit + deal.commission + deal.swap + deal.fee;
    }
    let account_shut = mt5.account_info().await?;
    println!(
        "round trip cost {realised:.5} {} — balance {} -> {}",
        account.currency, account.balance, account_shut.balance
    );
    println!(
        "balance moved by {:.5}",
        account_shut.balance - account.balance
    );
    Ok(())
}

/// Build a market order the contract will accept.
fn market(symbol: &SymbolInfo, side: u32, lot: f64, price: f64) -> TradeRequest {
    let mut request = TradeRequest::market(&symbol.name, side, lot, price)
        .deviation(20)
        .magic(MAGIC)
        .comment("mt5-rs roundtrip");
    if let Some(mode) = symbol.preferred_filling() {
        request = request.filling(mode);
    }
    request
}

/// Among the contracts this account may actually trade right now, the one
/// whose spread costs least for one minimum lot — that is what this example
/// pays.
///
/// "Right now" is the part worth spelling out: a stock keeps its last quote
/// long after its exchange has shut, so the cheapest spread on the list is
/// often a market that will answer `MARKET_CLOSED`. The newest quote time
/// across all symbols is the server's clock, and a symbol whose own quote is
/// far behind it is not trading.
async fn cheapest_round_trip(mt5: &Mt5, account: &AccountInfo) -> mt5::Result<Option<SymbolInfo>> {
    let all = mt5.symbols(None).await?;
    let server_now = all.iter().map(|s| s.time).max().unwrap_or(0);
    let mut best: Option<(f64, SymbolInfo)> = None;
    for symbol in all {
        if symbol.trade_mode != TRADE_MODE_FULL || symbol.bid <= 0.0 || symbol.ask <= symbol.bid {
            continue;
        }
        if server_now - symbol.time > STALE_QUOTE {
            continue;
        }
        if symbol.trade_tick_size <= 0.0 || symbol.trade_tick_value_loss <= 0.0 {
            continue;
        }
        // Spread in ticks times what a tick is worth, for the smallest lot.
        let cost = (symbol.ask - symbol.bid) / symbol.trade_tick_size
            * symbol.trade_tick_value_loss
            * symbol.volume_min;
        let cheaper = match &best {
            Some((seen, _)) => cost < *seen,
            None => true,
        };
        if cost > 0.0 && cheaper {
            best = Some((cost, symbol));
        }
    }
    if let Some((cost, symbol)) = &best {
        println!(
            "cheapest of the tradeable contracts: {} at about {cost:.5} {} the round trip",
            symbol.name, account.currency
        );
    }
    Ok(best.map(|(_, symbol)| symbol))
}
