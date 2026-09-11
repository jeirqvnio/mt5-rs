//! Log an account in, then exercise the calls a trade-enabled account
//! unlocks. `MT5_LOGIN`, `MT5_PASSWORD`, `MT5_SERVER`, plus the relay env.
//! Demo accounts only: a pending order is placed far from the market and
//! removed again.

use mt5::{order_type, retcode, Config, Mt5, TradeRequest};

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let config = Config::relay(
        &std::env::var("MT5_RELAY_ADDR").unwrap_or_else(|_| "127.0.0.1:18813".into()),
        &std::env::var("MT5_BRIDGE_TOKEN").unwrap_or_default(),
    )
    .connect_timeout(std::time::Duration::from_secs(30));
    let login: i64 = std::env::var("MT5_LOGIN")
        .unwrap_or_default()
        .parse()
        .unwrap_or(0);
    let password = std::env::var("MT5_PASSWORD").unwrap_or_default();
    let server = std::env::var("MT5_SERVER").unwrap_or_default();
    let sym = std::env::var("MT5_SYMBOL").unwrap_or_else(|_| "EURUSD".into());

    let mt5 = Mt5::connect(config).await?;
    println!(
        "before: {:?}",
        mt5.account_info().await.map(|a| (a.login, a.server))?
    );
    let started = std::time::Instant::now();
    match mt5.login(login, &password, &server).await {
        Ok(()) => println!("login ok in {:?}", started.elapsed()),
        Err(e) => println!("login ERR {e}"),
    }
    let a = mt5.account_info().await?;
    let ti = mt5.terminal_info().await?;
    println!(
        "terminal: connected {} algo_trading {}",
        ti.connected, ti.trade_allowed
    );
    println!(
        "after: {} {} balance {} {} trade_allowed {}",
        a.login, a.server, a.balance, a.currency, a.trade_allowed
    );
    match mt5.login(login, "wrong-password", &server).await {
        Ok(()) => println!("wrong password: accepted?!"),
        Err(e) => println!("wrong password: {e}"),
    }
    match mt5.login(login, &password, &server).await {
        Ok(()) => println!("re-login ok"),
        Err(e) => println!("re-login ERR {e}"),
    }

    mt5.symbol_select(&sym, true).await?;
    let s = mt5.symbol_info(&sym).await?;
    let t = mt5.symbol_tick(&sym).await?;
    println!(
        "calc_margin  {:?}",
        mt5.calc_margin(order_type::BUY, &sym, 1.0, t.ask).await
    );
    println!(
        "calc_profit  {:?}",
        mt5.calc_profit(order_type::BUY, &sym, 1.0, t.ask, t.ask + 0.001)
            .await
    );

    let mut market = TradeRequest::market(&sym, order_type::BUY, s.volume_min, t.ask)
        .deviation(20)
        .magic(7);
    if let Some(m) = s.preferred_filling() {
        market = market.filling(m);
    }
    let c = mt5.order_check(&market).await?;
    println!(
        "order_check  {} {} margin {} free {} {:?}",
        c.retcode,
        retcode::name(c.retcode),
        c.margin,
        c.margin_free,
        c.comment
    );

    // A buy limit 10% below the market cannot fill; it exercises order_send.
    let resting = s.normalize_price(t.bid * 0.9);
    let pending = TradeRequest::pending(&sym, order_type::BUY_LIMIT, s.volume_min, resting)
        .magic(7)
        .comment("mt5-rs probe");
    let sent = mt5.order_send(&pending).await?;
    println!(
        "order_send   {} {} order {} {:?}",
        sent.retcode,
        retcode::name(sent.retcode),
        sent.order,
        sent.comment
    );
    if sent.is_success() {
        let mine = mt5.order(sent.order).await?;
        println!(
            "order(t)     {:?}",
            mine.as_ref()
                .map(|o| (o.ticket, o.kind, o.price_open, o.magic, o.comment.clone()))
        );
        if let Some(o) = mine {
            let moved = mt5
                .order_send(&TradeRequest::modify(&o, s.normalize_price(resting * 0.99)))
                .await?;
            println!(
                "modify       {} {}",
                moved.retcode,
                retcode::name(moved.retcode)
            );
        }
        let gone = mt5.order_send(&TradeRequest::remove(sent.order)).await?;
        println!(
            "remove       {} {}",
            gone.retcode,
            retcode::name(gone.retcode)
        );
        println!("orders now   {}", mt5.orders_total().await?);
        let now = t.time;
        println!(
            "history_order {:?}",
            mt5.history_order(sent.order)
                .await?
                .map(|o| (o.state, o.comment))
        );
        println!(
            "history_orders_total {}",
            mt5.history_orders_total(now - 86_400, now + 86_400).await?
        );
    }
    Ok(())
}
