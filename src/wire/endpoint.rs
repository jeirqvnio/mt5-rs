//! Where the terminal is, and a connected stream to it.

use std::fmt::Write as _;
use std::pin::Pin;
use std::task::{Context, Poll};
#[cfg(windows)]
use std::time::Duration;

use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient};
use tokio::net::TcpStream;

#[cfg(not(windows))]
use crate::error::Error;
use crate::error::Result;
use crate::wire::send_token;

/// Windows' "all pipe instances are busy": a wait, not a refusal. The
/// terminal serves exactly two instances; one frees when a client closes.
#[cfg(windows)]
const ERROR_PIPE_BUSY: i32 = 231;
#[cfg(windows)]
const PIPE_BUSY_WAIT: Duration = Duration::from_secs(5);
#[cfg(windows)]
const PIPE_BUSY_POLL: Duration = Duration::from_millis(50);

/// Where a terminal can be reached.
#[derive(Clone)]
pub enum Endpoint {
    /// The terminal's named pipe. Windows only — see [`pipe_name_for`].
    Pipe(String),
    /// `host:port` of an `mt5-relay` beside the terminal, and its shared secret.
    Relay { addr: String, token: String },
}

/// Redacted: the relay token grants trading access and must not reach a log.
impl std::fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::Pipe(name) => f.debug_tuple("Pipe").field(name).finish(),
            Endpoint::Relay { addr, .. } => f
                .debug_struct("Relay")
                .field("addr", addr)
                .field("token", &"<redacted>")
                .finish(),
        }
    }
}

impl std::fmt::Display for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Endpoint::Pipe(name) => f.write_str(name),
            Endpoint::Relay { addr, .. } => f.write_str(addr),
        }
    }
}

/// The pipe the terminal publishes for a given `terminal64.exe` path: SHA-256
/// of `\\?\` + the lowercased path as UTF-16LE, as uppercase hex.
pub fn pipe_name_for(terminal_path: &str) -> String {
    let seed = format!(r"\\?\{}", terminal_path.to_lowercase());
    let utf16: Vec<u8> = seed.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut name = String::from(r"\\.\pipe\MT5.Terminal.");
    for byte in Sha256::digest(&utf16) {
        let _ = write!(name, "{byte:02X}");
    }
    name
}

/// A connected byte stream, whichever endpoint it came from.
#[derive(Debug)]
pub enum Stream {
    Tcp(TcpStream),
    #[cfg(windows)]
    Pipe(NamedPipeClient),
}

impl Stream {
    /// Connect, and for a relay present the token too.
    pub async fn connect(endpoint: &Endpoint) -> Result<Self> {
        match endpoint {
            Endpoint::Relay { addr, token } => {
                let socket = TcpStream::connect(addr).await?;
                // Strict request/response on small frames; Nagle would add a
                // round trip to every call.
                socket.set_nodelay(true)?;
                let mut stream = Stream::Tcp(socket);
                send_token(&mut stream, token).await?;
                Ok(stream)
            }
            #[cfg(windows)]
            Endpoint::Pipe(name) => {
                let deadline = tokio::time::Instant::now() + PIPE_BUSY_WAIT;
                loop {
                    match ClientOptions::new().open(name) {
                        Ok(client) => return Ok(Stream::Pipe(client)),
                        Err(e)
                            if e.raw_os_error() == Some(ERROR_PIPE_BUSY)
                                && tokio::time::Instant::now() < deadline =>
                        {
                            tokio::time::sleep(PIPE_BUSY_POLL).await;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
            #[cfg(not(windows))]
            Endpoint::Pipe(name) => Err(Error::Protocol(format!(
                "{name}: named pipes exist only on Windows; from macOS or Linux run \
                 mt5-relay beside the terminal and use Endpoint::Relay"
            ))),
        }
    }
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            #[cfg(windows)]
            Stream::Pipe(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            #[cfg(windows)]
            Stream::Pipe(s) => Pin::new(s).poll_write(cx, buf),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_flush(cx),
            #[cfg(windows)]
            Stream::Pipe(s) => Pin::new(s).poll_flush(cx),
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Stream::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            #[cfg(windows)]
            Stream::Pipe(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pipe_name_for;

    #[test]
    fn the_pipe_name_is_derived_from_the_lowercased_path() {
        let a = pipe_name_for(r"C:\Program Files\MetaTrader 5\terminal64.exe");
        let b = pipe_name_for(r"c:\program files\metatrader 5\terminal64.exe");
        assert_eq!(a, b);
        assert_eq!(a.len(), r"\\.\pipe\MT5.Terminal.".len() + 64);
    }
}
