//! The terminal, the account, and logging one in.

use std::time::Duration;

use tokio::time::{timeout, Instant};

use crate::client::{commands, Credentials, Mt5};
use crate::codec;
use crate::error::{Error, Result};
use crate::types::{AccountInfo, TerminalInfo, TerminalVersion};
use crate::wire::{self, Stream, Writer};

/// How often to ask whether a login has reached the broker.
const LOGIN_POLL: Duration = Duration::from_millis(200);

impl Mt5 {
    /// The terminal's IPC API version, build number and release date.
    pub async fn version(&self) -> Result<TerminalVersion> {
        let body = Writer::new().u32(0).into_bytes();
        codec::version(&self.call("version", commands::VERSION, &body).await?)
    }

    /// The terminal's own state: whether it has a broker session
    /// (`connected`), whether algo trading is switched on (`trade_allowed`),
    /// its build, paths and limits.
    pub async fn terminal_info(&self) -> Result<TerminalInfo> {
        codec::terminal_info(
            &self
                .call("terminal_info", commands::TERMINAL_INFO, &[])
                .await?,
        )
    }

    /// The logged-in account: balance, equity, margin, leverage, currency and
    /// the broker's name.
    pub async fn account_info(&self) -> Result<AccountInfo> {
        codec::account_info(
            &self
                .call("account_info", commands::ACCOUNT_INFO, &[])
                .await?,
        )
    }

    /// Log an account in on this session.
    ///
    /// The reply carries no verdict, so this waits for
    /// `terminal_info.connected`, up to `connect_timeout`.
    ///
    /// One-off: a reconnect logs in whatever [`crate::Config::account`] names,
    /// or nothing. For an account that must survive reconnects, put it in
    /// the configuration instead.
    pub async fn login(&self, login: i64, password: &str, server: &str) -> Result<()> {
        let credentials = Credentials {
            login,
            password: password.to_string(),
            server: server.to_string(),
        };
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            *guard = Some(self.handshake().await?);
        }
        let session = guard.as_mut().ok_or(Error::NotConnected)?;
        let outcome = log_in(
            &mut session.stream,
            &credentials,
            self.config.connect_timeout,
        )
        .await;
        if outcome.is_err() {
            // A login that did not finish may leave a reply unread.
            *guard = None;
        }
        outcome
    }
}

/// Send the login and wait for the broker session.
///
/// The terminal acknowledges a wrong password exactly like a right one, and
/// only `terminal_info.connected` tells them apart.
pub(crate) async fn log_in(
    stream: &mut Stream,
    credentials: &Credentials,
    waited: Duration,
) -> Result<()> {
    let body = Writer::new()
        .u32(0)
        .i64(credentials.login)
        .string(&credentials.password)
        .string(&credentials.server)
        .into_bytes();
    let failed = || Error::LoginFailed {
        login: credentials.login,
        server: credentials.server.clone(),
        waited,
    };
    let deadline = Instant::now() + waited;
    let attempt = async {
        wire::request(&mut *stream, commands::LOGIN, &body).await?;
        loop {
            let raw = wire::request(&mut *stream, commands::TERMINAL_INFO, &[]).await?;
            if codec::terminal_info(&raw)?.connected {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(failed());
            }
            tokio::time::sleep(LOGIN_POLL).await;
        }
    };
    timeout(waited, attempt)
        .await
        .unwrap_or_else(|_| Err(failed()))
}
