# mt5

MetaTrader 5 client for Rust. Speaks the terminal's native IPC protocol
directly. No Python.

Requires Rust 1.75 or later and a running MetaTrader 5 terminal.

## Install

```toml
[dependencies]
mt5 = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Usage

```rust
use mt5::{order_type, Mt5, TradeRequest};

#[tokio::main]
async fn main() -> mt5::Result<()> {
    let mt5 = Mt5::relay("127.0.0.1:18813", "token").await?;

    let tick = mt5.symbol_tick("EURUSD").await?;
    println!("{} / {}", tick.bid, tick.ask);

    let request = TradeRequest::market("EURUSD", order_type::BUY, 0.1, tick.ask)
        .deviation(20)
        .magic(42);
    let result = mt5.order_send(&request).await?;
    println!("{}", mt5::retcode::name(result.retcode));
    Ok(())
}
```

## Connecting

The terminal listens on a Windows named pipe.

On Windows, open it directly:

```rust
let mt5 = Mt5::terminal(r"C:\Program Files\MetaTrader 5\terminal64.exe").await?;
```

On macOS and Linux the terminal runs under Wine, where the pipe is not
visible to native processes. Run `mt5-relay` inside the Wine prefix and
connect to it over TCP:

```rust
let mt5 = Mt5::relay("127.0.0.1:18813", &token).await?;
```

Build the relay once:

```sh
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64            # Debian/Ubuntu: apt install gcc-mingw-w64-x86-64
cargo build --release --features relay --target x86_64-pc-windows-gnu
```

Start it under the Wine that owns the terminal. For the official macOS app:

```sh
export WINEPREFIX="$HOME/Library/Application Support/net.metaquotes.wine.metatrader5"
export MT5_BRIDGE_TOKEN=$(openssl rand -hex 16)
"/Applications/MetaTrader 5.app/Contents/SharedSupport/wine/bin/wine64" \
    target/x86_64-pc-windows-gnu/release/mt5-relay.exe
```

Relay configuration:

| variable | default | meaning |
|---|---|---|
| `MT5_BRIDGE_TOKEN` | required | shared secret clients must present |
| `MT5_PATH` | `C:\Program Files\MetaTrader 5\terminal64.exe` | terminal path, used to derive the pipe name |
| `MT5_PIPE_NAME` | derived from `MT5_PATH` | pipe name, if known |
| `MT5_RELAY_BIND` | `127.0.0.1` | listen address |
| `MT5_RELAY_PORT` | `18813` | listen port |

The port gives full trading access to the account. Bind it to localhost.

## Logging in

```rust
mt5.login(1_000_000, "password", "Broker-Demo").await?;
```

The protocol returns no verdict for a login. A wrong password is
acknowledged like a correct one and the terminal then fails to connect.
`login` polls `terminal_info().connected` and returns `Error::LoginFailed`
on timeout.

## API

All methods are on `Mt5`.

| group | methods |
|---|---|
| terminal | `version`, `terminal_info`, `account_info`, `login`, `build`, `disconnect` |
| symbols | `symbols_total`, `symbols`, `symbol_info`, `symbol_tick`, `symbol_select` |
| bars | `rates_from_pos`, `rates_from`, `rates_range` |
| ticks | `ticks_from`, `ticks_range` |
| positions | `positions`, `position`, `positions_total` |
| orders | `orders`, `order`, `orders_total` |
| history | `history_orders`, `history_order`, `history_orders_of_position`, `history_deals`, `history_deal`, `history_deals_of_position`, `history_orders_total`, `history_deals_total` |
| book | `book_add`, `book_get`, `book_release` |
| trading | `calc_margin`, `calc_profit`, `order_check`, `order_send` |

`TradeRequest` constructors: `market`, `pending`, `close`, `close_part`,
`protect`, `modify`, `remove`.

`TradeRequest` builders: `sl`, `tp`, `clear_sl`, `clear_tp`, `magic`,
`deviation`, `filling`, `comment`.

`SymbolInfo` helpers: `normalize_volume`, `normalize_price`,
`min_stop_distance`, `preferred_filling`, `supports_filling`.

Enumerations are constant modules with MQL5 values: `order_type`,
`trade_action`, `order_filling`, `order_time`, `position_type`, `deal_type`,
`deal_entry`, `filling_mask`, `copy_ticks`, `retcode`.

### Example

```rust
let info = mt5.symbol_info("EURUSD").await?;
let tick = mt5.symbol_tick("EURUSD").await?;

let mut request = TradeRequest::market("EURUSD", order_type::BUY, 0.1, tick.ask)
    .sl(tick.ask - 0.0020)
    .tp(tick.ask + 0.0040)
    .deviation(20)
    .magic(42);
if let Some(mode) = info.preferred_filling() {
    request = request.filling(mode);
}

let check = mt5.order_check(&request).await?;
if !check.is_ok() {
    return Ok(());
}
let result = mt5.order_send(&request).await?;
```

### Stop and target

They are independent. `sl` and `tp` set one each, chain both to set both,
and zero is how the protocol spells "no level".

```rust
let stop_only = TradeRequest::market("EURUSD", order_type::BUY, 0.1, tick.ask)
    .sl(tick.ask - 0.0020);
```

`protect` and `modify` start from the levels the position or order already
has, so changing one leaves the other alone. Both actions replace every level
on the server, so removing one is explicit:

```rust
let move_the_stop = TradeRequest::protect(&position).sl(1.1000);
let drop_the_target = TradeRequest::protect(&position).clear_tp();
let move_the_order = TradeRequest::modify(&order, 1.0850);
```

### Tick polling

MT5 has no push API. Poll from a mark and advance it:

```rust
let mut mark = 0;
loop {
    let batch = mt5.ticks_from("EURUSD", mark, 2_000, mt5::copy_ticks::ALL).await?;
    for tick in mt5::advance_ticks(&mut mark, batch) {
        // ...
    }
}
```

## Behaviour notes

- `ticks_from` and `ticks_range` take milliseconds (`time_msc`). Seconds are
  accepted but return data from hours earlier. A mark of 0 returns the newest
  ticks. The response includes the mark itself; `advance_ticks` removes it.
- `order_check` reports success as retcode 0, not `DONE`. Use `is_ok()`.
- `order_send` is never retried. On transport failure it returns
  `Error::OutcomeUnknown` and drops the connection. Set `magic` on every
  order and search `orders()` and `history_orders()` before resending. Read
  calls do reconnect and retry.
- Filling mode defaults to FOK. Brokers that do not support it reject every
  order with `INVALID_FILL` (10030). Use `preferred_filling()`.
- Algo trading must be enabled in the terminal or the server returns
  `CLIENT_DISABLES_AT` (10027). MT5 disables it on account change.
- `SymbolInfo::trade_mode` of 0 means quotes only; orders return
  `TRADE_DISABLED` (10017). 4 means full trading. Brokers commonly publish
  `EURUSD` for display and `EURUSD+` or `EURUSD.s` for trading.
- A terminal accepts two concurrent pipe clients. The third blocks, then
  fails.
- All timestamps are broker server time in Unix seconds. The terminal does
  not report its UTC offset. Use `symbol_tick().time` as the clock.

## Examples

```sh
export MT5_RELAY_ADDR=127.0.0.1:18813 MT5_BRIDGE_TOKEN=...
cargo run --example smoke       # read-only check of every call
cargo run --example probe       # every read method, with results
cargo run --example login       # login (MT5_LOGIN, MT5_PASSWORD, MT5_SERVER)
cargo run --example trade       # demo only: pending order, modify, remove
cargo run --example roundtrip   # demo only: market order, close, accounting
```

`trade` and `roundtrip` send real orders. `roundtrip` refuses to run on a
real account, uses the minimum lot and closes within a second.

## Protocol

The IPC protocol is undocumented. Command numbers and record layouts were
derived from the official Python module and verified against terminal builds
5836 to 6182.

Responses decode through a bounds-checked cursor that must consume each
record exactly. A changed field layout produces an error naming the field
rather than a wrong value. Trade requests are size-checked before sending.
There is no `unwrap`, `expect` or `panic!` outside tests; all three are
denied at the lint level.

Verified against a live terminal: all read calls, `order_check`,
`calc_margin` and `calc_profit` against `order_check` figures, pending order
send, modify and remove, and a market order filled and closed with the result
reconciled against deal history.

## License

MIT
