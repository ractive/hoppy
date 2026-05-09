//! Frame extraction for syslog-over-TCP streams.
//!
//! Bunny does not document which TCP framing it uses, and either is legal:
//!
//! * **Octet-counted** (RFC 6587 §3.4.1): `LEN SP MSG` — `LEN` is one or more
//!   ASCII digits, followed by a single space, then exactly `LEN` bytes of
//!   payload.
//! * **Non-transparent LF framing** (RFC 6587 §3.4.2): `MSG LF` — message
//!   terminated by a single newline. Embedded `LF` must be escaped by the
//!   sender; if it isn't, that's the sender's bug.
//!
//! We pick per-connection on the first byte:
//!
//! * `b'0'..=b'9'` → octet-counted
//! * `b'<'`        → LF-framed (RFC 5424 / 3164 messages always start with
//!   the priority value in angle brackets)
//! * anything else → [`FramingError::Malformed`] — caller logs and drops the
//!   connection.

use anyhow::Result;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

/// Maximum bytes accepted for a single octet-counted length prefix or a
/// single LF-framed line. RFC 5424 §6.1 caps a syslog message at 8 KiB but
/// recommends supporting up to 64 KiB; we go a little higher for headroom.
const MAX_FRAME_BYTES: usize = 128 * 1024;

/// Maximum number of ASCII digits we will read for the octet-counted length
/// prefix. `MAX_FRAME_BYTES` is < 1 000 000 → 6 digits is enough; we allow
/// 7 to give a clear error message instead of silently truncating.
const MAX_LENGTH_DIGITS: usize = 7;

/// Per-connection framing mode, locked in on the first byte of the stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// `LEN SP MSG` — RFC 6587 §3.4.1.
    OctetCounted,
    /// `MSG LF` — RFC 6587 §3.4.2.
    LfTerminated,
}

/// Errors a frame reader may surface to the caller.
#[derive(Debug, thiserror::Error)]
pub enum FramingError {
    /// First byte of the stream isn't a digit or `<` — we can't tell what
    /// the framing is supposed to be, so we bail. Caller should drop the
    /// connection (and probably log the byte for diagnosis).
    #[error("malformed frame: unexpected first byte 0x{0:02x}")]
    Malformed(u8),

    /// Octet-counted length prefix is longer than [`MAX_LENGTH_DIGITS`] —
    /// almost certainly a sender bug or a corrupt stream.
    #[error("octet-counted length prefix exceeds {MAX_LENGTH_DIGITS} digits")]
    LengthPrefixTooLong,

    /// Frame body is longer than [`MAX_FRAME_BYTES`] — guard against
    /// pathological / malicious input.
    #[error("frame exceeds maximum size of {MAX_FRAME_BYTES} bytes")]
    FrameTooLarge,

    /// The length prefix wasn't valid UTF-8 ASCII digits.
    #[error("invalid octet-counted length prefix: {0}")]
    InvalidLength(String),

    /// Underlying I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

// We only depend on `thiserror` for these errors. Add it to Cargo.toml if
// not already present; otherwise switch to a hand-rolled `Display` impl.

/// Detect the framing mode from the first byte without consuming it.
///
/// Returns `Ok(None)` on EOF (caller should close the connection cleanly),
/// `Err(FramingError::Malformed)` on an unexpected first byte.
pub async fn detect_framing<R>(reader: &mut R) -> Result<Option<Framing>, FramingError>
where
    R: AsyncBufRead + Unpin,
{
    let buf = reader.fill_buf().await?;
    let Some(&first) = buf.first() else {
        return Ok(None);
    };
    match first {
        b'0'..=b'9' => Ok(Some(Framing::OctetCounted)),
        b'<' => Ok(Some(Framing::LfTerminated)),
        other => Err(FramingError::Malformed(other)),
    }
}

/// Read the next frame from `reader` using the given `framing` mode.
///
/// Returns `Ok(None)` on a clean EOF between frames (so the listener can
/// reap the connection without surfacing an error). Returns
/// `Err(FramingError::Io)` on a partial frame at EOF.
pub async fn read_frame<R>(
    reader: &mut R,
    framing: Framing,
) -> Result<Option<Vec<u8>>, FramingError>
where
    R: AsyncBufRead + Unpin,
{
    match framing {
        Framing::OctetCounted => read_octet_counted(reader).await,
        Framing::LfTerminated => read_lf_terminated(reader).await,
    }
}

async fn read_octet_counted<R>(reader: &mut R) -> Result<Option<Vec<u8>>, FramingError>
where
    R: AsyncBufRead + Unpin,
{
    // Length prefix: ASCII digits up to a single space.
    let mut digits = Vec::with_capacity(MAX_LENGTH_DIGITS);
    loop {
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return if digits.is_empty() {
                    Ok(None)
                } else {
                    Err(FramingError::Io(e))
                };
            }
            Err(e) => return Err(FramingError::Io(e)),
        }
        match byte[0] {
            b' ' if !digits.is_empty() => break,
            d @ b'0'..=b'9' => {
                if digits.len() >= MAX_LENGTH_DIGITS {
                    return Err(FramingError::LengthPrefixTooLong);
                }
                digits.push(d);
            }
            other => {
                return Err(FramingError::InvalidLength(format!(
                    "unexpected byte 0x{other:02x} in length prefix"
                )));
            }
        }
    }
    let len_str =
        std::str::from_utf8(&digits).map_err(|e| FramingError::InvalidLength(e.to_string()))?;
    let len: usize = len_str
        .parse()
        .map_err(|e: std::num::ParseIntError| FramingError::InvalidLength(e.to_string()))?;
    if len > MAX_FRAME_BYTES {
        return Err(FramingError::FrameTooLarge);
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    Ok(Some(buf))
}

async fn read_lf_terminated<R>(reader: &mut R) -> Result<Option<Vec<u8>>, FramingError>
where
    R: AsyncBufRead + Unpin,
{
    let mut buf = Vec::with_capacity(512);
    let n = reader.read_until(b'\n', &mut buf).await?;
    if n == 0 {
        return Ok(None);
    }
    if buf.len() > MAX_FRAME_BYTES {
        return Err(FramingError::FrameTooLarge);
    }
    // Strip the trailing LF (and a CR if present, for CRLF tolerance).
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    if buf.last() == Some(&b'\r') {
        buf.pop();
    }
    if buf.is_empty() {
        // A bare LF — skip and keep reading.
        return Box::pin(read_lf_terminated(reader)).await;
    }
    Ok(Some(buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn detects_octet_counted_from_digit() {
        let data: &[u8] = b"42 hello";
        let mut r = BufReader::new(data);
        assert_eq!(
            detect_framing(&mut r).await.unwrap(),
            Some(Framing::OctetCounted)
        );
    }

    #[tokio::test]
    async fn detects_lf_from_angle_bracket() {
        let data: &[u8] = b"<14>foo\n";
        let mut r = BufReader::new(data);
        assert_eq!(
            detect_framing(&mut r).await.unwrap(),
            Some(Framing::LfTerminated)
        );
    }

    #[tokio::test]
    async fn detects_eof_on_empty_stream() {
        let data: &[u8] = b"";
        let mut r = BufReader::new(data);
        assert_eq!(detect_framing(&mut r).await.unwrap(), None);
    }

    #[tokio::test]
    async fn rejects_garbage_first_byte() {
        let data: &[u8] = b"garbage";
        let mut r = BufReader::new(data);
        let err = detect_framing(&mut r).await.unwrap_err();
        assert!(matches!(err, FramingError::Malformed(_)));
    }

    #[tokio::test]
    async fn reads_octet_counted_single_frame() {
        let data: &[u8] = b"5 hello";
        let mut r = BufReader::new(data);
        let frame = read_frame(&mut r, Framing::OctetCounted).await.unwrap();
        assert_eq!(frame.as_deref(), Some(&b"hello"[..]));
    }

    #[tokio::test]
    async fn reads_octet_counted_two_frames() {
        let data: &[u8] = b"5 hello3 abc";
        let mut r = BufReader::new(data);
        let f1 = read_frame(&mut r, Framing::OctetCounted).await.unwrap();
        let f2 = read_frame(&mut r, Framing::OctetCounted).await.unwrap();
        let f3 = read_frame(&mut r, Framing::OctetCounted).await.unwrap();
        assert_eq!(f1.as_deref(), Some(&b"hello"[..]));
        assert_eq!(f2.as_deref(), Some(&b"abc"[..]));
        assert_eq!(f3, None);
    }

    #[tokio::test]
    async fn reads_lf_terminated_single_frame() {
        let data: &[u8] = b"<14>1 - host app - - - hello\n";
        let mut r = BufReader::new(data);
        let frame = read_frame(&mut r, Framing::LfTerminated).await.unwrap();
        assert_eq!(frame.as_deref(), Some(&b"<14>1 - host app - - - hello"[..]));
    }

    #[tokio::test]
    async fn reads_lf_terminated_strips_crlf() {
        let data: &[u8] = b"<14>foo\r\n<14>bar\n";
        let mut r = BufReader::new(data);
        let f1 = read_frame(&mut r, Framing::LfTerminated).await.unwrap();
        let f2 = read_frame(&mut r, Framing::LfTerminated).await.unwrap();
        assert_eq!(f1.as_deref(), Some(&b"<14>foo"[..]));
        assert_eq!(f2.as_deref(), Some(&b"<14>bar"[..]));
    }

    #[tokio::test]
    async fn rejects_oversize_octet_counted_length_prefix() {
        let data: &[u8] = b"12345678 x";
        let mut r = BufReader::new(data);
        let err = read_frame(&mut r, Framing::OctetCounted).await.unwrap_err();
        assert!(matches!(err, FramingError::LengthPrefixTooLong));
    }

    #[tokio::test]
    async fn returns_none_on_clean_eof_between_frames() {
        let data: &[u8] = b"";
        let mut r = BufReader::new(data);
        let f = read_frame(&mut r, Framing::OctetCounted).await.unwrap();
        assert_eq!(f, None);
    }
}
