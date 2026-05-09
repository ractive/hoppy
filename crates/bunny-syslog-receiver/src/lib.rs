//! Embedded RFC 5424 / 3164 syslog TCP receiver used by
//! `hoppy container logs` to capture log-forwarding traffic from Bunny
//! Magic Containers.
//!
//! Two layers:
//!
//! * [`receiver`] — a `tokio` TCP listener that accepts framed syslog
//!   messages (RFC 6587 octet-counted **or** non-transparent LF framing,
//!   detected per-connection on the first byte) and forwards parsed
//!   [`LogEvent`]s through an `mpsc` channel.
//! * [`tunnel`] — a [`Tunnel`] trait + a [`BoreTunnel`] implementation
//!   that exposes the local listener via the `bore` CLI.
//!
//! The crate is transport-only: pretty-printing, redaction, and lifecycle
//! orchestration live in the consuming binary.

pub mod framing;
pub mod receiver;
pub mod tunnel;

pub use receiver::{LocalListener, LogEvent, Severity, run_receiver, spawn_receiver};
pub use tunnel::{BoreTunnel, NoopTunnel, StaticTunnel, Tunnel, TunnelHandle};
