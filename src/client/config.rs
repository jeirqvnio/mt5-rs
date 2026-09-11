//! Where to connect and how long to wait.

use std::time::Duration;

use crate::wire::{pipe_name_for, Endpoint};

/// How to reach a terminal, and how long to wait for it.
#[derive(Debug, Clone)]
pub struct Config {
    pub endpoint: Endpoint,
    /// Connect, handshake, and how long a login may take to reach the broker.
    /// A terminal that has never seen the broker first fetches the server
    /// list, which does not fit in five seconds.
    pub connect_timeout: Duration,
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
