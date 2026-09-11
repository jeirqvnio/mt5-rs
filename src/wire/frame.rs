//! One command exchange, and the raw frame the relay forwards.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::error::{Error, Result};
use crate::wire::Cursor;

const RESPONSE_HEADER: usize = 8;
/// A corrupt length prefix must not turn into a huge allocation.
pub const MAX_FRAME: usize = 256 * 1024 * 1024;

/// Send `command` with `body`; return the reply body after its 8-byte header.
pub async fn request<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    command: u32,
    body: &[u8],
) -> Result<Vec<u8>> {
    let payload_len = 4 + body.len();
    if payload_len > MAX_FRAME {
        return Err(Error::Protocol(format!(
            "command {command}: request too large"
        )));
    }
    let mut frame = Vec::with_capacity(8 + body.len());
    frame.extend_from_slice(&(payload_len as u32).to_le_bytes());
    frame.extend_from_slice(&command.to_le_bytes());
    frame.extend_from_slice(body);
    stream.write_all(&frame).await?;
    stream.flush().await?;

    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    let len = u32::from_le_bytes(header) as usize;
    if !(RESPONSE_HEADER..=MAX_FRAME).contains(&len) {
        return Err(Error::Protocol(format!(
            "command {command}: response length {len} out of range"
        )));
    }
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;

    let mut c = Cursor::new(&payload);
    let echoed = c.u32("echo")?;
    let status = c.u32("status")?;
    if echoed != command {
        // The stream is out of step; the caller drops the connection.
        return Err(Error::Protocol(format!(
            "sent command {command}, reply echoed {echoed}"
        )));
    }
    if status == 0 {
        return Err(Error::Refused(format!("command {command}")));
    }
    Ok(payload.split_off(RESPONSE_HEADER))
}

/// Read one length-prefixed frame, header included, into `buf`. `false` on a
/// clean end of stream.
pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R, buf: &mut Vec<u8>) -> Result<bool> {
    let mut header = [0u8; 4];
    match reader.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(false),
        Err(e) => return Err(e.into()),
    }
    let len = u32::from_le_bytes(header) as usize;
    if len > MAX_FRAME {
        return Err(Error::Protocol(format!(
            "frame of {len} bytes over the limit"
        )));
    }
    buf.clear();
    buf.extend_from_slice(&header);
    buf.resize(4 + len, 0);
    reader.read_exact(&mut buf[4..]).await?;
    Ok(true)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_request_is_framed_and_its_reply_unwrapped() {
        let (mut client, mut server) = tokio::io::duplex(4096);
        let peer = tokio::spawn(async move {
            let mut frame = Vec::new();
            assert!(read_frame(&mut server, &mut frame).await.unwrap());
            assert_eq!(frame, [6, 0, 0, 0, 190, 0, 0, 0, 1, 2]);
            let reply = [12, 0, 0, 0, 190, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0];
            server.write_all(&reply).await.unwrap();
        });
        assert_eq!(
            request(&mut client, 190, &[1, 2]).await.unwrap(),
            [7, 0, 0, 0]
        );
        peer.await.unwrap();
    }

    #[tokio::test]
    async fn a_zero_status_is_a_refusal_and_a_wrong_echo_a_desync() {
        let (mut client, mut server) = tokio::io::duplex(4096);
        server
            .write_all(&[8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0])
            .await
            .unwrap();
        assert!(matches!(
            request(&mut client, 1, &[]).await,
            Err(Error::Refused(_))
        ));
        server
            .write_all(&[8, 0, 0, 0, 9, 0, 0, 0, 1, 0, 0, 0])
            .await
            .unwrap();
        assert!(matches!(
            request(&mut client, 1, &[]).await,
            Err(Error::Protocol(_))
        ));
    }
}
