//! Tunnel abstraction for exposing the local syslog listener to a public
//! endpoint that Bunny's log-forwarding service can reach.
//!
//! The default implementation, [`BoreTunnel`], shells out to the `bore` CLI
//! (`cargo install bore-cli` / `brew install bore-cli`). We deliberately do
//! **not** vendor or bundle bore — it's a separately maintained tool with
//! its own release cadence, and forcing a copy of it into this binary
//! would add maintenance burden for no clear benefit.
//!
//! Two escape-hatch implementations:
//!
//! * [`NoopTunnel`] — `--tunnel none`. Returns the local listener address
//!   without exposing it. Useful for users who already have a tunnel /
//!   ingress (corporate VPN, public IP, etc.).
//! * [`StaticTunnel`] — `--tunnel-host host:port`. The user has already
//!   established forwarding (typically `ssh -R`) and just needs hoppy to
//!   register that public address with Bunny.
//!
//! All three implement [`Tunnel`]. A future `NgrokTunnel` slots in
//! cleanly.

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

/// Public endpoint exposed by a tunnel.
#[derive(Debug)]
pub struct TunnelHandle {
    /// Public hostname or IP that Bunny should send syslog traffic to.
    pub public_host: String,
    /// Public TCP port.
    pub public_port: u16,
    /// Per-implementation cleanup state. Dropping this terminates the
    /// tunnel; `stop().await` waits for clean shutdown.
    inner: TunnelInner,
}

#[derive(Debug)]
enum TunnelInner {
    /// Child process running `bore local …`.
    BoreChild(Child),
    /// User-managed tunnel — nothing to clean up.
    External,
}

impl TunnelHandle {
    /// Wait for the tunnel to terminate cleanly. For [`BoreTunnel`] this
    /// kills the child process and reaps it; for [`NoopTunnel`] /
    /// [`StaticTunnel`] it's a no-op.
    pub async fn stop(mut self) -> Result<()> {
        match &mut self.inner {
            TunnelInner::BoreChild(child) => {
                // Best-effort: ignore failures here — the child may already
                // have exited (the `wait_for_exit` race in the CLI may have
                // reaped it first).
                let _ = child.start_kill();
                let _ = child.wait().await;
            }
            TunnelInner::External => {}
        }
        Ok(())
    }

    /// For [`BoreTunnel`]: borrow the child so the caller can race
    /// `child.wait()` against the receiver loop and surface unexpected
    /// tunnel deaths. Returns `None` for non-child tunnels.
    pub fn child_mut(&mut self) -> Option<&mut Child> {
        match &mut self.inner {
            TunnelInner::BoreChild(c) => Some(c),
            TunnelInner::External => None,
        }
    }
}

/// A pluggable public-ingress provider.
#[async_trait::async_trait]
pub trait Tunnel: Send + Sync {
    /// Open a tunnel from `local_port` to a public endpoint.
    async fn start(&self, local_port: u16) -> Result<TunnelHandle>;
}

/// Spawn the `bore` CLI to expose `local_port` via `bore.pub` (or a
/// caller-specified self-hosted bore server).
#[derive(Debug, Clone)]
pub struct BoreTunnel {
    /// Bore relay host (default: `bore.pub`).
    pub server: String,
    /// How long to wait for bore to print its `listening at …` line before
    /// giving up.
    pub startup_timeout: Duration,
    /// Override the binary name — used in tests with a fake `bore`.
    pub binary: String,
}

impl Default for BoreTunnel {
    fn default() -> Self {
        Self {
            server: "bore.pub".to_owned(),
            startup_timeout: Duration::from_secs(15),
            binary: "bore".to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl Tunnel for BoreTunnel {
    async fn start(&self, local_port: u16) -> Result<TunnelHandle> {
        let mut child = Command::new(&self.binary)
            .arg("local")
            .arg(local_port.to_string())
            .arg("--to")
            .arg(&self.server)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    anyhow!(
                        "`{}` not found on PATH. Install with `cargo install bore-cli` or \
                         `brew install bore-cli`, or pass `--tunnel none` / \
                         `--tunnel-host <host:port>` to skip bore.",
                        self.binary
                    )
                } else {
                    anyhow::Error::from(e).context(format!("failed to spawn `{}`", self.binary))
                }
            })?;

        let stdout = child
            .stdout
            .take()
            .context("bore child has no stdout pipe")?;
        let stderr = child
            .stderr
            .take()
            .context("bore child has no stderr pipe")?;

        // Read until we see the `listening at <host>:<port>` banner. Bore
        // prints it via `tracing` to stderr by default, but older versions
        // wrote to stdout — try stderr first, then fall back to stdout.
        let banner_fut = read_bore_banner(BufReader::new(stderr), BufReader::new(stdout));
        let (public_host, public_port) = match timeout(self.startup_timeout, banner_fut).await {
            Ok(Ok(addr)) => addr,
            Ok(Err(e)) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                return Err(e.context("bore failed to report a public address"));
            }
            Err(_) => {
                let _ = child.start_kill();
                let _ = child.wait().await;
                bail!(
                    "bore did not announce a public address within {:?}",
                    self.startup_timeout
                );
            }
        };

        Ok(TunnelHandle {
            public_host,
            public_port,
            inner: TunnelInner::BoreChild(child),
        })
    }
}

/// Watch both stderr (bore ≥ 0.5 uses tracing → stderr) and stdout (older
/// versions). Whichever yields the `listening at host:port` line first
/// wins.
async fn read_bore_banner<R1, R2>(
    mut stderr: BufReader<R1>,
    mut stdout: BufReader<R2>,
) -> Result<(String, u16)>
where
    R1: tokio::io::AsyncRead + Unpin,
    R2: tokio::io::AsyncRead + Unpin,
{
    let mut stderr_done = false;
    let mut stdout_done = false;
    let mut err_line = String::new();
    let mut out_line = String::new();
    loop {
        if stderr_done && stdout_done {
            bail!("bore exited before printing a public address");
        }
        err_line.clear();
        out_line.clear();
        tokio::select! {
            r = stderr.read_line(&mut err_line), if !stderr_done => match r {
                Ok(0) => stderr_done = true,
                Ok(_) => {
                    if let Some(addr) = parse_bore_banner(&err_line) {
                        return Ok(addr);
                    }
                }
                Err(e) => return Err(e.into()),
            },
            r = stdout.read_line(&mut out_line), if !stdout_done => match r {
                Ok(0) => stdout_done = true,
                Ok(_) => {
                    if let Some(addr) = parse_bore_banner(&out_line) {
                        return Ok(addr);
                    }
                }
                Err(e) => return Err(e.into()),
            },
        }
    }
}

/// Parse a line like `listening at bore.pub:12345` (current bore) or
/// older `listening at bore.pub:12345 …` variants. Returns `None` if the
/// line is unrelated.
fn parse_bore_banner(line: &str) -> Option<(String, u16)> {
    // Match against the substring after "listening at " — bore prefixes the
    // banner with timestamps / log levels in newer releases, so we can't
    // anchor on the start.
    let needle = "listening at ";
    let idx = line.find(needle)?;
    let rest = &line[idx + needle.len()..];
    // Take the host:port token (whitespace-delimited, may have trailing
    // ANSI codes / punctuation).
    let token = rest
        .split_whitespace()
        .next()?
        .trim_end_matches(['.', ',', ';']);
    let (host, port) = token.rsplit_once(':')?;
    let port: u16 = port.parse().ok()?;
    Some((host.to_owned(), port))
}

/// `--tunnel none`: return the local address as-is.
#[derive(Debug, Clone)]
pub struct NoopTunnel {
    pub local_host: String,
}

impl Default for NoopTunnel {
    fn default() -> Self {
        Self {
            local_host: "127.0.0.1".to_owned(),
        }
    }
}

#[async_trait::async_trait]
impl Tunnel for NoopTunnel {
    async fn start(&self, local_port: u16) -> Result<TunnelHandle> {
        Ok(TunnelHandle {
            public_host: self.local_host.clone(),
            public_port: local_port,
            inner: TunnelInner::External,
        })
    }
}

/// `--tunnel-host <host:port>`: user has already opened a tunnel; we just
/// register the address with Bunny and listen locally.
#[derive(Debug, Clone)]
pub struct StaticTunnel {
    pub host: String,
    pub port: u16,
}

#[async_trait::async_trait]
impl Tunnel for StaticTunnel {
    async fn start(&self, _local_port: u16) -> Result<TunnelHandle> {
        Ok(TunnelHandle {
            public_host: self.host.clone(),
            public_port: self.port,
            inner: TunnelInner::External,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bore_banner_simple() {
        assert_eq!(
            parse_bore_banner("listening at bore.pub:38291\n"),
            Some(("bore.pub".to_owned(), 38291))
        );
    }

    #[test]
    fn parses_bore_banner_with_log_prefix() {
        let line = "2026-05-09T10:00:00.000Z  INFO bore_cli: listening at bore.pub:7000\n";
        assert_eq!(parse_bore_banner(line), Some(("bore.pub".to_owned(), 7000)));
    }

    #[test]
    fn unrelated_line_does_not_match() {
        assert_eq!(parse_bore_banner("connecting...\n"), None);
        assert_eq!(parse_bore_banner(""), None);
    }

    #[tokio::test]
    async fn noop_tunnel_returns_local_addr() {
        let t = NoopTunnel::default();
        let h = t.start(45678).await.unwrap();
        assert_eq!(h.public_host, "127.0.0.1");
        assert_eq!(h.public_port, 45678);
    }

    #[tokio::test]
    async fn static_tunnel_returns_user_addr() {
        let t = StaticTunnel {
            host: "vps.example.com".to_owned(),
            port: 5514,
        };
        let h = t.start(0).await.unwrap();
        assert_eq!(h.public_host, "vps.example.com");
        assert_eq!(h.public_port, 5514);
    }

    #[tokio::test]
    async fn bore_missing_binary_yields_install_hint() {
        let t = BoreTunnel {
            binary: "definitely-not-a-real-binary-9f3a7c".to_owned(),
            ..Default::default()
        };
        let err = t.start(12345).await.unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("not found on PATH"), "got: {msg}");
        assert!(msg.contains("cargo install bore-cli"), "got: {msg}");
    }
}
