//! Forwards TCP to the terminal's named pipe, from inside the Windows session
//! that owns it (real Windows, Wine, or a container). Frames go through
//! unchanged; the only thing added is a token handshake on the socket.
//!
//! ```sh
//! cargo build --release --features relay --target x86_64-pc-windows-gnu
//! MT5_BRIDGE_TOKEN=secret wine target/x86_64-pc-windows-gnu/release/mt5-relay.exe
//! ```
//!
//! Environment: `MT5_BRIDGE_TOKEN` (required), `MT5_PATH` (default
//! `C:\Program Files\MetaTrader 5\terminal64.exe`) or `MT5_PIPE_NAME`,
//! `MT5_RELAY_BIND` (default `127.0.0.1`), `MT5_RELAY_PORT` (default `18813`).
//!
//! The terminal serves two pipe instances; the relay opens one per client.

#[cfg(windows)]
mod relay {
    use std::env;
    use std::process::ExitCode;

    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    use mt5::wire::{expect_token, read_frame, Endpoint, Stream};
    use mt5::{pipe_name_for, Error, Result};

    const DEFAULT_PATH: &str = r"C:\Program Files\MetaTrader 5\terminal64.exe";

    pub async fn main() -> ExitCode {
        match run().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("mt5-relay: {err}");
                ExitCode::FAILURE
            }
        }
    }

    async fn run() -> Result<()> {
        let pipe = match env::var("MT5_PIPE_NAME") {
            Ok(name) if !name.is_empty() => name,
            _ => pipe_name_for(&env::var("MT5_PATH").unwrap_or_else(|_| DEFAULT_PATH.to_string())),
        };
        // This port reaches a terminal that can trade.
        let token = env::var("MT5_BRIDGE_TOKEN")
            .ok()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| Error::Unauthorized("MT5_BRIDGE_TOKEN is required".into()))?;
        let addr = format!(
            "{}:{}",
            env::var("MT5_RELAY_BIND").unwrap_or_else(|_| "127.0.0.1".into()),
            env::var("MT5_RELAY_PORT").unwrap_or_else(|_| "18813".into())
        );
        let listener = TcpListener::bind(&addr).await?;
        eprintln!("mt5-relay: listening on {addr} -> {pipe}");
        loop {
            let (socket, peer) = listener.accept().await?;
            let pipe = pipe.clone();
            let token = token.clone();
            tokio::spawn(async move {
                if let Err(err) = serve(socket, &pipe, &token).await {
                    eprintln!("mt5-relay: {peer} ended: {err}");
                }
            });
        }
    }

    async fn serve(mut socket: TcpStream, pipe: &str, token: &str) -> Result<()> {
        socket.set_nodelay(true)?;
        expect_token(&mut socket, token).await?;
        let mut pipe = Stream::connect(&Endpoint::Pipe(pipe.to_string())).await?;
        let mut buf = Vec::new();
        loop {
            if !read_frame(&mut socket, &mut buf).await? {
                return Ok(());
            }
            pipe.write_all(&buf).await?;
            pipe.flush().await?;
            if !read_frame(&mut pipe, &mut buf).await? {
                return Err(Error::Protocol("the terminal closed the pipe".into()));
            }
            socket.write_all(&buf).await?;
            socket.flush().await?;
        }
    }
}

#[cfg(windows)]
#[tokio::main]
async fn main() -> std::process::ExitCode {
    relay::main().await
}

#[cfg(not(windows))]
fn main() -> std::process::ExitCode {
    eprintln!("mt5-relay runs inside the Windows session that owns the terminal; build it with --target x86_64-pc-windows-gnu and run it under Wine");
    std::process::ExitCode::FAILURE
}
