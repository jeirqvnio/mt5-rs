//! The terminal, the account, and logging one in.

use std::time::Duration;

use tokio::time::{timeout, Instant};

use crate::client::{commands, Mt5};
use crate::codec;
use crate::error::{Error, Result};
use crate::types::{AccountInfo, TerminalInfo, TerminalVersion};
use crate::wire::{self, Writer};

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

    /// Log an account in. The reply carries no verdict — a wrong password is
    /// acknowledged like a right one — so this waits for
    /// `terminal_info.connected`, up to `connect_timeout`.
    pub async fn login(&self, login: i64, password: &str, server: &str) -> Result<()> {
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            *guard = Some(self.handshake().await?);
        }
        let session = guard.as_mut().ok_or(Error::NotConnected)?;
        let body = Writer::new()
            .u32(0)
            .i64(login)
            .string(password)
            .string(server)
            .into_bytes();
        let waited = self.config.connect_timeout;
        let failed = || Error::LoginFailed {
            login,
            server: server.to_string(),
            waited,
        };
        let deadline = Instant::now() + waited;
        let outcome = timeout(waited, async {
            wire::request(&mut session.stream, commands::LOGIN, &body).await?;
            loop {
                let raw = wire::request(&mut session.stream, commands::TERMINAL_INFO, &[]).await?;
                if codec::terminal_info(&raw)?.connected {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(failed());
                }
                tokio::time::sleep(LOGIN_POLL).await;
            }
        })
        .await;
        match outcome {
            Ok(result) => result,
            Err(_) => {
                *guard = None;
                Err(failed())
            }
        }
    }
}
