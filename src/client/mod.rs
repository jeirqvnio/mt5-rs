//! The client: one connection, one command in flight, every call bounded.
//!
//! Creating a client touches nothing. The first call, or an explicit
//! [`Mt5::connect`], opens the session. Reads are retried across a reconnect.
//! Trade commands are issued exactly once: a transport failure there is
//! [`Error::OutcomeUnknown`], and the connection is dropped so a late reply
//! cannot be read as the next call's.
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

pub use config::{Config, Credentials};

/// Client name sent in the handshake; the terminal logs it.
const CLIENT_NAME: &str = "Go";

#[derive(Debug)]
struct Session {
    stream: Stream,
    build: u32,
}

/// A MetaTrader 5 terminal.
///
/// `Send + Sync`: share one behind an `Arc`, or keep several in a `Vec` for
/// parallel pipes to the same terminal. A terminal serves two.
// ponytail: one connection, one in-flight command — the pipe has no
// correlation id, so concurrency means a second connection, not a smarter
// client.
#[derive(Debug)]
pub struct Mt5 {
    config: Config,
    session: Mutex<Option<Session>>,
}

impl Mt5 {
    /// A client that has not connected yet. Nothing touches the network until
    /// [`Mt5::connect`] or the first call.
    pub fn new(config: Config) -> Self {
        Mt5 {
            config,
            session: Mutex::new(None),
        }
    }

    /// Connect through a relay and open the session now.
    pub async fn relay(addr: &str, token: &str) -> Result<Self> {
        let mt5 = Mt5::new(Config::relay(addr, token));
        mt5.connect().await?;
        Ok(mt5)
    }

    /// Connect to the terminal installed at this path and open the session
    /// now. Windows only.
    pub async fn terminal(terminal64_path: &str) -> Result<Self> {
        let mt5 = Mt5::new(Config::terminal(terminal64_path));
        mt5.connect().await?;
        Ok(mt5)
    }

    /// Open the session if there is none, so a bad endpoint or a refused
    /// login fails here and not on the first call.
    ///
    /// Idempotent: on a live session this does nothing. A session that broke
    /// has already been dropped by the call that saw it break, so calling
    /// this after an error reopens it. To force a new session on one that
    /// still works, [`Mt5::disconnect`] first.
    pub async fn connect(&self) -> Result<()> {
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            *guard = Some(self.handshake().await?);
        }
        Ok(())
    }

    /// The endpoint and timeouts this client was built with.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Terminal build from the handshake, or `None` before connecting.
    pub async fn build(&self) -> Option<u32> {
        self.session.lock().await.as_ref().map(|s| s.build)
    }

    /// Drop the connection. The next call reconnects on its own, so this is
    /// for releasing one of the terminal's two pipe slots, or for forcing a
    /// fresh session with [`Mt5::connect`].
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
        if let Some(credentials) = &self.config.account {
            account::log_in(&mut stream, credentials, deadline).await?;
        }
        Ok(Session { stream, build })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared<T: Send + Sync + 'static>() {}

    /// Lives in a `Vec`, behind an `Arc`, inside `tokio::spawn`.
    #[test]
    fn a_client_can_be_shared_across_tasks() {
        shared::<Mt5>();
    }

    #[tokio::test]
    async fn creating_a_client_touches_nothing() {
        // Nothing listens on port 9; a client that connected in `new` would
        // have nowhere to go. This one must not have tried.
        let mt5 = Mt5::new(Config::relay("127.0.0.1:9", "token"));
        assert_eq!(mt5.build().await, None);
        assert!(mt5.connect().await.is_err(), "connect is where it fails");
    }
}
