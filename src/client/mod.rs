//! The client: one connection, one command in flight, every call bounded.
//!
//! Reads are retried across a reconnect. Trade commands are issued exactly
//! once: a transport failure there is [`Error::OutcomeUnknown`], and the
//! connection is dropped so a late reply cannot be read as the next call's.
//!
//! The API is split by area: `account`, `symbols`, `market`, `positions`,
//! `trade` — each an `impl Mt5` block in its own file.

mod account;
mod call;
mod commands;
mod config;
mod market;
mod positions;
mod symbols;
mod trade;

use tokio::sync::Mutex;
use tokio::time::timeout;

use crate::codec;
use crate::error::{Error, Result};
use crate::wire::{self, Stream, Writer};

pub use config::Config;

/// Client name sent in the handshake; the terminal logs it.
const CLIENT_NAME: &str = "Go";

#[derive(Debug)]
struct Session {
    stream: Stream,
    build: u32,
}

/// A MetaTrader 5 terminal.
// ponytail: one connection, one in-flight command — the pipe has no
// correlation id, so concurrency means a second connection, not a smarter
// client. The terminal allows two.
#[derive(Debug)]
pub struct Mt5 {
    config: Config,
    session: Mutex<Option<Session>>,
}

impl Mt5 {
    /// Connect through a relay (macOS, Linux, Docker, another machine).
    pub async fn relay(addr: &str, token: &str) -> Result<Self> {
        Mt5::connect(Config::relay(addr, token)).await
    }

    /// Connect to the terminal installed at this path. Windows only.
    pub async fn terminal(terminal64_path: &str) -> Result<Self> {
        Mt5::connect(Config::terminal(terminal64_path)).await
    }

    /// Connect eagerly, so a bad endpoint fails here and not on the first call.
    pub async fn connect(config: Config) -> Result<Self> {
        let mt5 = Mt5 {
            config,
            session: Mutex::new(None),
        };
        *mt5.session.lock().await = Some(mt5.handshake().await?);
        Ok(mt5)
    }

    /// The endpoint and timeouts this client was built with.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Terminal build from the handshake.
    pub async fn build(&self) -> Option<u32> {
        self.session.lock().await.as_ref().map(|s| s.build)
    }

    /// Drop the connection. The next call reconnects on its own, so this is
    /// for releasing one of the terminal's two pipe slots, not for shutdown.
    pub async fn disconnect(&self) {
        self.session.lock().await.take();
    }

    async fn handshake(&self) -> Result<Session> {
        let deadline = self.config.connect_timeout;
        let mut stream = timeout(deadline, Stream::connect(&self.config.endpoint))
            .await
            .map_err(|_| Error::Timeout {
                method: "connect",
                elapsed: deadline,
            })??;
        let hello = Writer::new()
            .u32(commands::SESSION_HELLO)
            .string(CLIENT_NAME)
            .into_bytes();
        let reply = timeout(
            deadline,
            wire::request(&mut stream, commands::SESSION, &hello),
        )
        .await
        .map_err(|_| Error::Timeout {
            method: "handshake",
            elapsed: deadline,
        })??;
        let build = codec::count(&reply, "handshake.build")?;
        Ok(Session { stream, build })
    }
}
