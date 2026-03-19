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
