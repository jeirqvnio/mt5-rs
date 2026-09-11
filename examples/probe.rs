//! Call every read-only method once and report what came back. Nothing is
//! sent to a book: `order_check` validates, `order_send` is not called.
//! Same environment as `smoke`.

use std::fmt::Debug;

use mt5::{copy_ticks, Config, Mt5, OrderType, Timeframe, TradeRequest};

fn report<T: Debug>(name: &str, r: mt5::Result<T>) {
    match r {
        Ok(v) => {
            let text = format!("{v:?}");
            let cut = text.chars().take(110).collect::<String>();
            println!("ok   {name:<28} {cut}");
        }
        Err(e) => println!("ERR  {name:<28} {e}"),
    }
}

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let config = match std::env::var("MT5_PATH") {
        Ok(path) => Config::terminal(&path),
        Err(_) => Config::relay(
            &std::env::var("MT5_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18813".into()),
            &std::env::var("MT5_BRIDGE_TOKEN").unwrap_or_default(),
        ),
    };
    let sym = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "EURUSD".into());
    let mt5 = Mt5::connect(config).await?;
    report("build", Ok(mt5.build().await));
    report("version", mt5.version().await);
    report(
        "terminal_info",
        mt5.terminal_info()
            .await
            .map(|t| (t.build, t.connected, t.trade_allowed, t.company)),
    );
    report(
        "account_info",
        mt5.account_info()
            .await
            .map(|a| (a.login, a.server, a.balance, a.currency)),
    );

    report("symbols_total", mt5.symbols_total().await);
    report("symbols(None)", mt5.symbols(None).await.map(|v| v.len()));
    report(
        "symbols(*USD*)",
        mt5.symbols(Some("*USD*"))
            .await
            .map(|v| v.iter().map(|s| s.name.clone()).take(5).collect::<Vec<_>>()),
    );
    report("symbol_select", mt5.symbol_select(&sym, true).await);
    report(
        "symbol_info",
        mt5.symbol_info(&sym).await.map(|s| {
            (
                s.name,
                s.digits,
                s.volume_step,
                s.filling_mode,
                s.currency_profit,
            )
        }),
    );
    report(
        "symbol_info(bogus)",
        mt5.symbol_info("NOPE123").await.map(|s| s.name),
    );
    let tick = mt5.symbol_tick(&sym).await;
    report(
        "symbol_tick",
        tick.as_ref()
            .map(|t| (t.time, t.bid, t.ask, t.time_msc))
            .map_err(|e| mt5::Error::Protocol(e.to_string())),
    );
    let now = tick.as_ref().map(|t| t.time).unwrap_or(0);
    let now_ms = tick.as_ref().map(|t| t.time_msc).unwrap_or(0);

    report(
        "rates_from_pos",
        mt5.rates_from_pos(&sym, Timeframe::M1, 0, 3)
            .await
            .map(|v| v.iter().map(|r| (r.time, r.close)).collect::<Vec<_>>()),
    );
    report(
        "rates_from",
        mt5.rates_from(&sym, Timeframe::H1, now - 7200, 2)
            .await
            .map(|v| v.iter().map(|r| r.time).collect::<Vec<_>>()),
    );
    report(
        "rates_range",
        mt5.rates_range(&sym, Timeframe::M5, now - 1800, now)
            .await
            .map(|v| v.len()),
    );
    report(
        "ticks_from(0)",
        mt5.ticks_from(&sym, 0, 3, copy_ticks::ALL)
            .await
            .map(|v| v.iter().map(|t| t.time_msc).collect::<Vec<_>>()),
    );
    report(
        "ticks_from(mark)",
        mt5.ticks_from(&sym, now_ms - 60_000, 1000, copy_ticks::INFO)
            .await
            .map(|v| v.len()),
    );
    report(
        "ticks_range",
        mt5.ticks_range(&sym, now_ms - 60_000, now_ms, copy_ticks::ALL)
            .await
            .map(|v| v.len()),
    );

    report("positions_total", mt5.positions_total().await);
    report(
        "positions(None)",
        mt5.positions(None).await.map(|v| v.len()),
    );
    report(
        "positions(sym)",
        mt5.positions(Some(&sym)).await.map(|v| v.len()),
    );
    report(
        "position(1)",
        mt5.position(1).await.map(|p| p.map(|p| p.ticket)),
    );
    report("orders_total", mt5.orders_total().await);
    report("orders(None)", mt5.orders(None).await.map(|v| v.len()));
    report("orders(sym)", mt5.orders(Some(&sym)).await.map(|v| v.len()));
    report("order(1)", mt5.order(1).await.map(|o| o.map(|o| o.ticket)));

    let (from, to) = (now - 30 * 86_400, now + 3_600);
    report(
        "history_orders_total",
        mt5.history_orders_total(from, to).await,
    );
    let orders = mt5.history_orders(from, to).await;
    report(
        "history_orders",
        orders
            .as_ref()
            .map(|v| v.len())
            .map_err(|e| mt5::Error::Protocol(e.to_string())),
    );
    let sample = orders
        .ok()
        .and_then(|v| v.first().map(|o| (o.ticket, o.position_id)));
    report(
        "history_deals_total",
        mt5.history_deals_total(from, to).await,
    );
    let deals = mt5.history_deals(from, to).await;
    report(
        "history_deals",
        deals
            .as_ref()
            .map(|v| {
                v.iter()
                    .map(|d| (d.ticket, d.kind, d.profit))
                    .take(3)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| mt5::Error::Protocol(e.to_string())),
    );
    let deal = deals
        .ok()
        .and_then(|v| v.first().map(|d| (d.ticket, d.position_id)));
    let (ticket, position) = sample.or(deal).unwrap_or((1, 1));
    report(
        "history_order(t)",
        mt5.history_order(ticket)
            .await
            .map(|o| o.map(|o| (o.ticket, o.symbol))),
    );
    report(
        "history_deal(t)",
        mt5.history_deal(deal.map_or(1, |d| d.0))
            .await
            .map(|d| d.map(|d| (d.ticket, d.kind))),
    );
    report(
        "history_orders_of_pos",
        mt5.history_orders_of_position(position)
            .await
            .map(|v| v.len()),
    );
    report(
        "history_deals_of_pos",
        mt5.history_deals_of_position(position)
            .await
            .map(|v| v.len()),
    );

    report("book_add", mt5.book_add(&sym).await);
    report("book_get", mt5.book_get(&sym).await.map(|v| v.len()));
    report("book_release", mt5.book_release(&sym).await);

    if let Ok(t) = &tick {
        let s = mt5.symbol_info(&sym).await?;
        let mut r = TradeRequest::market(&sym, OrderType::Buy, s.volume_min, t.ask)
            .deviation(20)
            .magic(7);
        if let Some(m) = s.preferred_filling() {
            r = r.filling(m);
        }
        report(
            "order_check",
            mt5.order_check(&r).await.map(|c| (c.retcode, c.comment)),
        );
        report(
            "calc_margin",
            mt5.calc_margin(OrderType::Buy, &sym, 1.0, t.ask).await,
        );
        report(
            "calc_profit",
            mt5.calc_profit(OrderType::Buy, &sym, 1.0, t.ask, t.ask + 0.001)
                .await,
        );
    }
    report(
        "terminal_info(again)",
        mt5.terminal_info().await.map(|t| t.connected),
    );
    mt5.disconnect().await;
    report("after disconnect", mt5.symbols_total().await);
    Ok(())
}
