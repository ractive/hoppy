use assert_cmd::Command;

/// Build a hoppy Command with all BUNNY_* env vars cleared.
pub fn hoppy_cmd() -> Command {
    let mut cmd = Command::cargo_bin("hoppy").expect("binary not found");
    cmd.env_remove("BUNNY_API_KEY");
    cmd.env_remove("BUNNY_API_URL");
    cmd.env_remove("BUNNY_STORAGE_URL");
    cmd.env_remove("BUNNY_STREAM_URL");
    cmd.env_remove("BUNNY_CONTAINERS_URL");
    cmd.env_remove("BUNNY_STORAGE_KEY");
    cmd.env_remove("BUNNY_STREAM_KEY");
    cmd
}

/// Build a hoppy Command pointed at a mock server for the core API.
pub fn hoppy_mock_cmd(api_key: &str, core_url: &str) -> Command {
    let mut cmd = hoppy_cmd();
    cmd.env("BUNNY_API_KEY", api_key);
    cmd.env("BUNNY_API_URL", core_url);
    cmd
}

/// Variant that also sets storage/stream/containers URLs.
pub fn hoppy_mock_cmd_full(
    api_key: &str,
    core_url: &str,
    storage_url: Option<&str>,
    stream_url: Option<&str>,
    containers_url: Option<&str>,
) -> Command {
    let mut cmd = hoppy_mock_cmd(api_key, core_url);
    if let Some(url) = storage_url {
        cmd.env("BUNNY_STORAGE_URL", url);
        cmd.env("BUNNY_STORAGE_KEY", "mock-storage-key");
    }
    if let Some(url) = stream_url {
        cmd.env("BUNNY_STREAM_URL", url);
        cmd.env("BUNNY_STREAM_KEY", "mock-stream-key");
    }
    if let Some(url) = containers_url {
        cmd.env("BUNNY_CONTAINERS_URL", url);
    }
    cmd
}

/// Load a fixture file from the shared fixtures/ directory.
pub fn fixture(relative_path: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative_path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {relative_path}: {e}"))
}

#[cfg(feature = "live-api")]
pub fn hoppy_live_cmd() -> Command {
    let mut cmd = Command::cargo_bin("hoppy").expect("binary not found");
    assert!(
        std::env::var("BUNNY_API_KEY").is_ok(),
        "BUNNY_API_KEY required"
    );
    cmd
}

#[cfg(feature = "live-api")]
pub fn unique_name(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{ts}-{n}")
}

// ---------------------------------------------------------------------------
// Live API test helpers
// ---------------------------------------------------------------------------

/// Result of a live hoppy command.
#[cfg(feature = "live-api")]
#[allow(dead_code)]
pub struct LiveResult {
    pub json: Option<serde_json::Value>,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// Run `hoppy --format json <args>` against the live API and parse JSON output.
#[cfg(feature = "live-api")]
#[allow(dead_code)]
pub fn hoppy_live_json(args: &[&str]) -> LiveResult {
    let output = hoppy_live_cmd()
        .args(["--format", "json"])
        .args(args)
        .output()
        .expect("failed to run hoppy");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let json = serde_json::from_str(&stdout).ok();
    LiveResult {
        json,
        stdout,
        stderr,
        success: output.status.success(),
    }
}

/// Run `hoppy --yes <args>` against the live API (raw, no JSON parsing).
#[cfg(feature = "live-api")]
#[allow(dead_code)]
pub fn hoppy_live_raw(args: &[&str]) -> LiveResult {
    let output = hoppy_live_cmd()
        .arg("--yes")
        .args(args)
        .output()
        .expect("failed to run hoppy");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    LiveResult {
        json: None,
        stdout,
        stderr,
        success: output.status.success(),
    }
}

/// Run `hoppy --yes --format json <args>` against the live API.
#[cfg(feature = "live-api")]
#[allow(dead_code)]
pub fn hoppy_live_json_yes(args: &[&str]) -> LiveResult {
    let output = hoppy_live_cmd()
        .args(["--yes", "--format", "json"])
        .args(args)
        .output()
        .expect("failed to run hoppy");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let json = serde_json::from_str(&stdout).ok();
    LiveResult {
        json,
        stdout,
        stderr,
        success: output.status.success(),
    }
}

/// Cleanup stack — collects delete commands and runs them in reverse order.
/// Uses best-effort: failures are printed but don't propagate.
#[cfg(feature = "live-api")]
pub struct CleanupStack {
    actions: Vec<Vec<String>>,
}

#[cfg(feature = "live-api")]
impl CleanupStack {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    /// Push a cleanup command (args to pass to hoppy --yes).
    pub fn push(&mut self, args: &[&str]) {
        self.actions
            .push(args.iter().map(|s| s.to_string()).collect());
    }

    /// Run all cleanup commands in reverse order (best-effort).
    pub fn run(&self) {
        for args in self.actions.iter().rev() {
            let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let result = hoppy_live_raw(&str_args);
            if !result.success {
                eprintln!(
                    "cleanup warning: `hoppy {}` failed: {}",
                    args.join(" "),
                    result.stderr.trim()
                );
            }
        }
    }
}

/// Run a lifecycle test with panic-safe cleanup.
/// The closure receives a mutable `CleanupStack` to register teardown actions.
/// Even if the closure panics, cleanup runs.
#[cfg(feature = "live-api")]
pub fn run_lifecycle<F>(f: F)
where
    F: FnOnce(&mut CleanupStack) + std::panic::UnwindSafe,
{
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let cleanup = AssertUnwindSafe(std::cell::RefCell::new(CleanupStack::new()));
    let result = catch_unwind(|| {
        f(&mut cleanup.borrow_mut());
    });
    cleanup.borrow().run();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
