//! Where to connect, as whom, and how long to wait.

use std::fmt;
use std::time::Duration;

use crate::wire::{pipe_name_for, Endpoint};

/// How to reach a terminal, and how long to wait for it.
#[derive(Debug, Clone)]
pub struct Config {
    pub endpoint: Endpoint,
    /// Logged in on every connection, including the reconnects a read makes
    /// on its own. `None` leaves the terminal on whatever account it holds.
    pub account: Option<Credentials>,
    /// Connect, handshake, and how long a login may take to reach the broker.
    /// A terminal that has never seen the broker first fetches the server
    /// list, which does not fit in five seconds.
    pub connect_timeout: Duration,
    /// How long one read may take before it counts as failed and is retried.
    pub request_timeout: Duration,
    /// Generous on purpose: a premature timeout turns a fill into an unknown.
    pub trade_timeout: Duration,
    /// Attempts for a read, reconnecting between them.
    pub retries: u32,
}

impl Config {
    /// A configuration with the default timeouts for an endpoint you built
    /// yourself. Prefer [`Config::relay`] or [`Config::terminal`].
    pub fn new(endpoint: Endpoint) -> Self {
        Config {
            endpoint,
            account: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            trade_timeout: Duration::from_secs(60),
            retries: 3,
        }
    }

    /// A relay at `host:port` and the token it was started with.
    pub fn relay(addr: &str, token: &str) -> Self {
        Config::new(Endpoint::Relay {
            addr: addr.to_string(),
            token: token.to_string(),
        })
    }

    /// The pipe of the terminal installed at `terminal64_path`. Windows only.
    pub fn terminal(terminal64_path: &str) -> Self {
        Config::new(Endpoint::Pipe(pipe_name_for(terminal64_path)))
    }

    /// Log this account in on every connection.
    ///
    /// Every handshake, including the one a read makes after a dropped
    /// connection, logs in and waits for the broker session before any call
    /// goes through. Logging in to the account the terminal already holds
    /// returns within milliseconds.
    ///
    /// The cost of that guarantee: while the terminal cannot reach the broker,
    /// a reconnect waits `connect_timeout` for a login that does not complete
    /// and then fails, so reads fail too until the broker is back.
    pub fn account(mut self, login: i64, password: &str, server: &str) -> Self {
        self.account = Some(Credentials {
            login,
            password: password.to_string(),
            server: server.to_string(),
        });
        self
    }

    /// How long connecting, the handshake and a login may each take.
    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }
    /// How long one read may take before it counts as failed and is retried.
    pub fn request_timeout(mut self, d: Duration) -> Self {
        self.request_timeout = d;
        self
    }
    /// How long a trade command may take. Keep it generous: a premature
    /// timeout turns a fill into an outcome nobody knows.
    pub fn trade_timeout(mut self, d: Duration) -> Self {
        self.trade_timeout = d;
        self
    }
}

/// A trading account on a broker's server.
#[derive(Clone, PartialEq, Eq)]
pub struct Credentials {
    pub login: i64,
    pub password: String,
    pub server: String,
}

/// Redacted: a configuration gets logged, and a password must not go with it.
impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Credentials")
            .field("login", &self.login)
            .field("password", &"<redacted>")
            .field("server", &self.server)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_logged_configuration_carries_no_secret() {
        let config = Config::relay("127.0.0.1:18813", "relay-secret").account(
            1_000_000,
            "account-secret",
            "Broker-Demo",
        );
        let printed = format!("{config:?}");
        assert!(!printed.contains("relay-secret"), "{printed}");
        assert!(!printed.contains("account-secret"), "{printed}");
        assert!(printed.contains("1000000") && printed.contains("Broker-Demo"));
        assert!(printed.contains("127.0.0.1:18813"));
    }
}
