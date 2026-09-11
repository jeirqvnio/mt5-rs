//! How a command reaches the terminal: the retry discipline for reads, the
//! single attempt for trades, and the three reply shapes.

use std::time::Duration;

use tokio::time::{timeout, Instant};

use crate::client::Mt5;
use crate::codec;
use crate::error::{Error, Result};
use crate::wire;

const BACKOFF_BASE: Duration = Duration::from_millis(50);
const BACKOFF_CAP: Duration = Duration::from_secs(2);

impl Mt5 {
    /// A read: retried across a reconnect on transport failures.
    pub(crate) async fn call(
        &self,
        method: &'static str,
        command: u32,
        body: &[u8],
    ) -> Result<Vec<u8>> {
        let mut guard = self.session.lock().await;
        let mut last = Error::NotConnected;
        for attempt in 0..self.config.retries.max(1) {
            if attempt > 0 {
                let backoff = (BACKOFF_BASE * 2u32.saturating_pow(attempt - 1)).min(BACKOFF_CAP);
                tokio::time::sleep(backoff).await;
            }
            if guard.is_none() {
                match self.handshake().await {
                    Ok(s) => *guard = Some(s),
                    Err(e) => {
                        last = e;
                        continue;
                    }
                }
            }
            let Some(session) = guard.as_mut() else {
                continue;
            };
            let started = Instant::now();
            let exchange = wire::request(&mut session.stream, command, body);
            let failure = match timeout(self.config.request_timeout, exchange).await {
                Ok(Ok(payload)) => return Ok(payload),
                Ok(Err(e)) => e,
                Err(_) => Error::Timeout {
                    method,
                    elapsed: started.elapsed(),
                },
            };
            // A half-finished exchange leaves bytes in the pipe; the session
            // goes rather than the next call decoding them.
            *guard = None;
            if !failure.is_transient() {
                return Err(failure);
            }
            last = failure;
        }
        Err(last)
    }

    /// A trade command: exactly one attempt, never retried.
    pub(crate) async fn call_once(
        &self,
        method: &'static str,
        command: u32,
        body: &[u8],
    ) -> Result<Vec<u8>> {
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            *guard = Some(self.handshake().await?);
        }
        let Some(session) = guard.as_mut() else {
            return Err(Error::NotConnected);
        };
        let started = Instant::now();
        let exchange = wire::request(&mut session.stream, command, body);
        let failure = match timeout(self.config.trade_timeout, exchange).await {
            Ok(Ok(payload)) => return Ok(payload),
            Ok(Err(e)) => e,
            Err(_) => Error::Timeout {
                method,
                elapsed: started.elapsed(),
            },
        };
        *guard = None;
        Err(Error::OutcomeUnknown {
            method,
            source: Box::new(failure),
        })
    }

    /// A reply that is one `u32`.
    pub(crate) async fn call_u32(
        &self,
        method: &'static str,
        command: u32,
        body: &[u8],
    ) -> Result<u32> {
        codec::count(&self.call(method, command, body).await?, method)
    }

    /// An acknowledgement: empty body, or a status word.
    pub(crate) async fn call_ack(
        &self,
        method: &'static str,
        command: u32,
        body: &[u8],
    ) -> Result<bool> {
        let raw = self.call(method, command, body).await?;
        Ok(raw.is_empty() || codec::count(&raw, method)? != 0)
    }

    /// A lookup that answers with one record or an empty body.
    pub(crate) async fn call_opt<T>(
        &self,
        method: &'static str,
        command: u32,
        body: &[u8],
        decode: fn(&[u8]) -> Result<T>,
    ) -> Result<Option<T>> {
        let raw = self.call(method, command, body).await?;
        if raw.is_empty() {
            Ok(None)
        } else {
            decode(&raw).map(Some)
        }
    }
}
