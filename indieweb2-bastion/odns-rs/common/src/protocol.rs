// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <jonathan.jewell@open.ac.uk>
//
// DNS-over-TLS framing: 2-byte big-endian length prefix + payload (RFC 7858 Section 3.3).

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Maximum DNS message size (including KEM overhead).
/// Kyber-1024 CT (1568) + nonce (24) + max DNS (65535) + tag (16) = ~67143
/// We cap at 64 KiB for the payload itself.
pub const MAX_PAYLOAD: usize = 65535;

/// Read a length-prefixed message from a stream.
///
/// Format: `[2 bytes: big-endian length] [N bytes: payload]`
pub async fn read_framed<R: AsyncRead + Unpin>(stream: &mut R) -> std::io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 2];
    stream.read_exact(&mut len_buf).await?;
    let len = u16::from_be_bytes(len_buf) as usize;

    if len == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "zero-length message",
        ));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Write a length-prefixed message to a stream.
///
/// Format: `[2 bytes: big-endian length] [N bytes: payload]`
pub async fn write_framed<W: AsyncWrite + Unpin>(
    stream: &mut W,
    data: &[u8],
) -> std::io::Result<()> {
    if data.len() > MAX_PAYLOAD {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("payload too large: {} > {}", data.len(), MAX_PAYLOAD),
        ));
    }

    let len = (data.len() as u16).to_be_bytes();
    stream.write_all(&len).await?;
    stream.write_all(data).await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn framed_roundtrip() {
        let data = b"hello, oDNS";
        let mut buf = Vec::new();
        write_framed(&mut buf, data).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let result = read_framed(&mut cursor).await.unwrap();
        assert_eq!(result, data);
    }

    #[tokio::test]
    async fn rejects_zero_length() {
        let wire = [0u8, 0]; // length = 0
        let mut cursor = std::io::Cursor::new(wire.to_vec());
        let result = read_framed(&mut cursor).await;
        assert!(result.is_err());
    }
}
