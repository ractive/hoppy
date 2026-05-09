//! TCP syslog receiver.
//!
//! Bind a kernel-assigned port (or a caller-chosen one), accept connections,
//! detect framing on the first byte, and forward parsed [`LogEvent`]s on an
//! `mpsc` channel. Shutdown is cooperative: drop the
//! [`tokio_util::sync::CancellationToken`] passed to [`run_receiver`] (or
//! hold it via [`spawn_receiver`] and call `.cancel()` on the returned
//! handle).

use std::net::SocketAddr;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::framing::{Framing, FramingError, detect_framing, read_frame};

/// Syslog severity levels per RFC 5424 §6.2.1. Mirrored as a plain enum so
/// downstream code can pattern-match without depending on `syslog_loose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Informational,
    Debug,
}

impl Severity {
    fn from_loose(s: syslog_loose::SyslogSeverity) -> Self {
        use syslog_loose::SyslogSeverity::*;
        match s {
            SEV_EMERG => Severity::Emergency,
            SEV_ALERT => Severity::Alert,
            SEV_CRIT => Severity::Critical,
            SEV_ERR => Severity::Error,
            SEV_WARNING => Severity::Warning,
            SEV_NOTICE => Severity::Notice,
            SEV_INFO => Severity::Informational,
            SEV_DEBUG => Severity::Debug,
        }
    }

    /// Short fixed-width label for pretty printing (5 chars wide).
    pub fn label(self) -> &'static str {
        match self {
            Severity::Emergency => "EMERG",
            Severity::Alert => "ALERT",
            Severity::Critical => "CRIT ",
            Severity::Error => "ERROR",
            Severity::Warning => "WARN ",
            Severity::Notice => "NOTIC",
            Severity::Informational => "INFO ",
            Severity::Debug => "DEBUG",
        }
    }
}

/// One parsed syslog message, ready for pretty-printing or JSON emission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    /// Original RFC 3339 / 5424 timestamp string, when the sender provided
    /// one. `None` for RFC 3164 messages without a year (we don't
    /// fabricate a timestamp).
    pub timestamp: Option<String>,
    pub hostname: Option<String>,
    pub app_name: Option<String>,
    pub proc_id: Option<String>,
    pub msg_id: Option<String>,
    pub severity: Option<Severity>,
    /// Free-form message body. May be empty.
    pub message: String,
}

impl LogEvent {
    /// Build a [`LogEvent`] from a raw frame payload. Returns `None` if the
    /// frame doesn't parse as syslog at all (caller logs and drops it).
    pub fn from_frame(bytes: &[u8]) -> Option<Self> {
        let s = std::str::from_utf8(bytes).ok()?;
        // `Variant::Either` lets the parser pick between RFC 3164 and 5424
        // per-message — Bunny may emit either depending on the operator's
        // forwarding-config `format` choice.
        let msg = syslog_loose::parse_message(s, syslog_loose::Variant::Either);
        // `syslog_loose` is deliberately permissive — it'll happily produce
        // a `Message` with no severity and the entire input as `msg` when
        // the frame is gibberish. Treat "no priority" as "couldn't parse".
        msg.severity?;
        Some(Self {
            timestamp: msg.timestamp.map(|t| t.to_rfc3339()),
            hostname: msg.hostname.map(str::to_owned),
            app_name: msg.appname.map(str::to_owned),
            proc_id: msg.procid.map(|p| p.to_string()),
            msg_id: msg.msgid.map(str::to_owned),
            severity: msg.severity.map(Severity::from_loose),
            message: msg.msg.to_owned(),
        })
    }
}

/// Handle to a bound listener. Holding this keeps the socket open and lets
/// callers learn the local address before spawning the accept loop.
#[derive(Debug)]
pub struct LocalListener {
    listener: TcpListener,
    addr: SocketAddr,
}

impl LocalListener {
    /// Bind to `127.0.0.1:port`. Pass port `0` for a kernel-assigned port.
    pub async fn bind(port: u16) -> Result<Self> {
        let bind_addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let listener = TcpListener::bind(bind_addr)
            .await
            .with_context(|| format!("failed to bind syslog listener on {bind_addr}"))?;
        let addr = listener
            .local_addr()
            .context("failed to read bound socket address")?;
        Ok(Self { listener, addr })
    }

    /// Bind to `0.0.0.0:port` for cases where the listener must accept
    /// connections from a tunnel running on a separate interface.
    pub async fn bind_all_interfaces(port: u16) -> Result<Self> {
        let bind_addr: SocketAddr = ([0, 0, 0, 0], port).into();
        let listener = TcpListener::bind(bind_addr)
            .await
            .with_context(|| format!("failed to bind syslog listener on {bind_addr}"))?;
        let addr = listener
            .local_addr()
            .context("failed to read bound socket address")?;
        Ok(Self { listener, addr })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Run the accept loop until `cancel` is triggered.
///
/// Each accepted connection is handled on its own task; per-connection
/// frame errors are logged via `tracing::warn!` and the connection is
/// dropped, but the listener keeps running.
pub async fn run_receiver(
    listener: LocalListener,
    tx: mpsc::Sender<LogEvent>,
    cancel: CancellationToken,
) -> Result<()> {
    let LocalListener { listener, addr } = listener;
    tracing::debug!(%addr, "syslog receiver: accept loop started");
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::debug!("syslog receiver: cancelled, exiting accept loop");
                return Ok(());
            }
            res = listener.accept() => {
                match res {
                    Ok((stream, peer)) => {
                        let tx = tx.clone();
                        let cancel = cancel.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, tx, cancel).await {
                                tracing::warn!(%peer, error = %e, "syslog connection ended with error");
                            }
                        });
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "accept failed");
                    }
                }
            }
        }
    }
}

/// Spawn [`run_receiver`] on a background task. Returns the bound address
/// and a [`tokio::task::JoinHandle`] for the accept loop. The caller owns
/// the [`CancellationToken`] used to shut it down.
pub fn spawn_receiver(
    listener: LocalListener,
    tx: mpsc::Sender<LogEvent>,
    cancel: CancellationToken,
) -> (SocketAddr, tokio::task::JoinHandle<Result<()>>) {
    let addr = listener.local_addr();
    let handle = tokio::spawn(run_receiver(listener, tx, cancel));
    (addr, handle)
}

async fn handle_connection(
    stream: TcpStream,
    tx: mpsc::Sender<LogEvent>,
    cancel: CancellationToken,
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let framing = match detect_framing(&mut reader).await {
        Ok(Some(f)) => f,
        Ok(None) => return Ok(()),
        Err(e) => {
            tracing::warn!(error = %e, "rejecting connection: malformed first byte");
            return Ok(());
        }
    };
    tracing::debug!(?framing, "framing locked in for connection");

    loop {
        tokio::select! {
            _ = cancel.cancelled() => return Ok(()),
            frame = read_frame_resilient(&mut reader, framing) => {
                match frame {
                    Some(bytes) => {
                        match LogEvent::from_frame(&bytes) {
                            Some(event) => {
                                if tx.send(event).await.is_err() {
                                    // Receiver gone — caller has shut us down.
                                    return Ok(());
                                }
                            }
                            None => {
                                tracing::warn!(
                                    bytes = bytes.len(),
                                    "dropping unparsable syslog frame"
                                );
                            }
                        }
                    }
                    None => return Ok(()),
                }
            }
        }
    }
}

/// Read a single frame; on a recoverable framing error log a warning and
/// return `None` so the connection is dropped. (We can't realistically
/// resync the stream once framing is wrong.)
async fn read_frame_resilient<R>(reader: &mut R, framing: Framing) -> Option<Vec<u8>>
where
    R: AsyncBufReadExt + Unpin,
{
    match read_frame(reader, framing).await {
        Ok(opt) => opt,
        Err(FramingError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => None,
        Err(e) => {
            tracing::warn!(error = %e, "framing error, dropping connection");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    fn rfc5424_frame(message: &str) -> Vec<u8> {
        // <14> = facility 1 (user), severity 6 (info)
        let body = format!("<14>1 2026-05-09T10:00:00Z hostA app1 1234 ID47 - {message}");
        let mut out = format!("{} ", body.len()).into_bytes();
        out.extend_from_slice(body.as_bytes());
        out
    }

    fn rfc5424_lf_frame(message: &str) -> Vec<u8> {
        let body = format!("<14>1 2026-05-09T10:00:00Z hostA app1 1234 ID47 - {message}\n");
        body.into_bytes()
    }

    #[tokio::test]
    async fn parses_octet_counted_frames() {
        let listener = LocalListener::bind(0).await.unwrap();
        let addr = listener.local_addr();
        let (tx, mut rx) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let server = tokio::spawn(run_receiver(listener, tx, cancel_clone));

        let mut s = TcpStream::connect(addr).await.unwrap();
        let mut bytes = Vec::new();
        bytes.extend(rfc5424_frame("hello"));
        bytes.extend(rfc5424_frame("world"));
        s.write_all(&bytes).await.unwrap();
        s.shutdown().await.unwrap();

        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        assert_eq!(e1.message, "hello");
        assert_eq!(e2.message, "world");
        assert_eq!(e1.severity, Some(Severity::Informational));
        assert_eq!(e1.app_name.as_deref(), Some("app1"));
        assert_eq!(e1.hostname.as_deref(), Some("hostA"));

        cancel.cancel();
        let _ = server.await;
    }

    #[tokio::test]
    async fn parses_lf_terminated_frames() {
        let listener = LocalListener::bind(0).await.unwrap();
        let addr = listener.local_addr();
        let (tx, mut rx) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let server = tokio::spawn(run_receiver(listener, tx, cancel_clone));

        let mut s = TcpStream::connect(addr).await.unwrap();
        let mut bytes = Vec::new();
        bytes.extend(rfc5424_lf_frame("alpha"));
        bytes.extend(rfc5424_lf_frame("beta"));
        s.write_all(&bytes).await.unwrap();
        s.shutdown().await.unwrap();

        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        assert_eq!(e1.message, "alpha");
        assert_eq!(e2.message, "beta");

        cancel.cancel();
        let _ = server.await;
    }

    #[tokio::test]
    async fn malformed_first_byte_drops_connection_only() {
        let listener = LocalListener::bind(0).await.unwrap();
        let addr = listener.local_addr();
        let (tx, mut rx) = mpsc::channel(8);
        let cancel = CancellationToken::new();
        let cancel_clone = cancel.clone();
        let server = tokio::spawn(run_receiver(listener, tx, cancel_clone));

        // First connection: garbage. Should be dropped quietly.
        let mut bad = TcpStream::connect(addr).await.unwrap();
        bad.write_all(b"garbage\n").await.unwrap();
        bad.shutdown().await.unwrap();

        // Second connection: valid. Listener should still serve it.
        let mut good = TcpStream::connect(addr).await.unwrap();
        good.write_all(&rfc5424_lf_frame("after-bad"))
            .await
            .unwrap();
        good.shutdown().await.unwrap();

        let e = rx.recv().await.unwrap();
        assert_eq!(e.message, "after-bad");

        cancel.cancel();
        let _ = server.await;
    }

    #[test]
    fn unparsable_frame_returns_none() {
        // Pure garbage with no priority — `syslog_loose` produces a message
        // with severity = None, which we treat as unparsable.
        let evt = LogEvent::from_frame(b"this is not syslog at all");
        assert!(evt.is_none());
    }
}
