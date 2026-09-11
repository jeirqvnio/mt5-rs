//! The relay handshake: `u32 len | token`, answered by `u32 0`.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Error, Result};

const MAX_TOKEN: usize = 4096;

/// Client side. The relay closes the socket on a bad token rather than
/// replying, which arrives here as [`Error::Unauthorized`].
pub async fn send_token<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    token: &str,
) -> Result<()> {
    let mut frame = (token.len() as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(token.as_bytes());
    stream.write_all(&frame).await?;
    stream.flush().await?;
    let mut ack = [0u8; 4];
    stream.read_exact(&mut ack).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            Error::Unauthorized("the relay rejected the token".into())
        } else {
            Error::Io(e)
        }
    })?;
    if ack != [0; 4] {
        return Err(Error::Protocol(
            "relay answered the token with a non-empty frame".into(),
        ));
    }
    Ok(())
}

/// Relay side. Constant-time compare, no reply on a mismatch: a prober should
/// not learn it found the right port.
pub async fn expect_token<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    token: &str,
) -> Result<()> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    let len = u32::from_le_bytes(header) as usize;
    if len > MAX_TOKEN {
        return Err(Error::Unauthorized("token frame too large".into()));
    }
    let mut offered = vec![0u8; len];
    stream.read_exact(&mut offered).await?;
    let expected = token.as_bytes();
    let same = offered.len() == expected.len()
        && offered
            .iter()
            .zip(expected)
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0;
    if !same {
        return Err(Error::Unauthorized("bad token".into()));
    }
    stream.write_all(&0u32.to_le_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_right_token_is_acked_and_the_wrong_one_refused() {
        let (mut client, mut server) = tokio::io::duplex(4096);
        let relay = tokio::spawn(async move { expect_token(&mut server, "s3cret").await });
        send_token(&mut client, "s3cret").await.unwrap();
        relay.await.unwrap().unwrap();

        let (mut client, mut server) = tokio::io::duplex(4096);
        tokio::spawn(async move {
            let _ = expect_token(&mut server, "right").await;
            drop(server);
        });
        assert!(matches!(
            send_token(&mut client, "wrong").await,
            Err(Error::Unauthorized(_))
        ));
    }
}
