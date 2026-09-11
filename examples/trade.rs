//! On a demo account with AutoTrading on: find a symbol the server lets us
//! trade, then place, move and remove a pending order far from the market.
//! Same environment as `login`; `MT5_SYMBOLS` is a comma list to try.

use mt5::{Config, Mt5, OrderType, SymbolTradeMode, TradeRequest};

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let config = Config::relay(
        &std::env::var("MT5_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18813".into()),
        &std::env::var("MT5_BRIDGE_TOKEN").unwrap_or_default(),
    );
    let mt5 = Mt5::connect(config).await?;
    let ti = mt5.terminal_info().await?;
    let a = mt5.account_info().await?;
    println!(
        "{} {} algo_trading {} account.trade_allowed {}",
        a.login, a.server, ti.trade_allowed, a.trade_allowed
    );

    // A symbol outside Market Watch must be selected before symbol_info
    // answers for it.
    let tradeable: Vec<String> = mt5
        .symbols(None)
        .await?
        .into_iter()
        .filter(|s| s.trade_mode == SymbolTradeMode::Full && s.bid > 0.0)
        .map(|s| s.name)
        .collect();
    println!(
        "tradeable: {} e.g. {:?}",
        tradeable.len(),
        tradeable.iter().take(6).collect::<Vec<_>>()
    );
    let chosen = std::env::var("MT5_SYMBOLS").ok().unwrap_or_else(|| {
        tradeable
            .iter()
            .find(|n| n.contains("BTC"))
            .or(tradeable.first())
            .cloned()
            .unwrap_or_default()
    });
    for sym in chosen.split(',') {
        mt5.symbol_select(sym, true).await?;
        let Ok(s) = mt5.symbol_info(sym).await else {
            println!("{sym}: unknown");
            continue;
        };
        let t = mt5.symbol_tick(sym).await?;
        println!(
            "{sym}: trade_mode {} exemode {} filling {} stops_level {} bid {}",
            s.trade_mode, s.trade_exemode, s.filling_mode, s.trade_stops_level, t.bid
        );
        let mut market = TradeRequest::market(sym, OrderType::Buy, s.volume_min, t.ask)
            .deviation(20)
            .magic(7);
        if let Some(m) = s.preferred_filling() {
            market = market.filling(m);
        }
        let c = mt5.order_check(&market).await?;
        println!(
            "  order_check {} margin {} free {} {:?}",
            c.retcode, c.margin, c.margin_free, c.comment
        );
        if !c.is_ok() {
            continue;
        }
        let resting = s.normalize_price(t.bid * 0.9);
        let pending = TradeRequest::pending(sym, OrderType::BuyLimit, s.volume_min, resting)
            .magic(7)
            .comment("mt5-rs probe");
        let sent = mt5.order_send(&pending).await?;
        println!(
            "  order_send  {} order {} {:?}",
            sent.retcode, sent.order, sent.comment
        );
        if !sent.is_success() {
            continue;
        }
        let mine = mt5.order(sent.order).await?;
        println!(
            "  order(t)    {:?}",
            mine.as_ref()
                .map(|o| (o.ticket, o.kind, o.price_open, o.magic, o.comment.clone()))
        );
        if let Some(o) = &mine {
            let moved = mt5
                .order_send(&TradeRequest::modify(o, s.normalize_price(resting * 0.99)))
                .await?;
            println!("  modify      {}", moved.retcode);
            println!(
                "  price now   {:?}",
                mt5.order(o.ticket).await?.map(|o| o.price_open)
            );
        }
        let gone = mt5.order_send(&TradeRequest::remove(sent.order)).await?;
        println!("  remove      {}", gone.retcode);
        println!("  orders now  {}", mt5.orders_total().await?);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        println!(
            "  history_order {:?}",
            mt5.history_order(sent.order)
                .await?
                .map(|o| (o.state, o.comment))
        );
        break;
    }
    Ok(())
}
