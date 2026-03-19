use assert_cmd::Command;
use wiremock::MockServer;

/// Build a `Command` for the `hoppy` binary with all env vars pointing at the
/// mock server, or pass through real env vars when `HOPPY_E2E_LIVE=1`.
pub fn hoppy(mock: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("hoppy").expect("hoppy binary not found");

    if std::env::var("HOPPY_E2E_LIVE").as_deref() == Ok("1") {
        // Live mode — use real credentials from the environment.
        // Don't override any URL env vars.
    } else {
        let url = mock.uri();
        cmd.env("BUNNY_API_KEY", "test-api-key")
            .env("BUNNY_API_URL", &url)
            .env("BUNNY_CONTAINERS_URL", &url)
            .env("BUNNY_STREAM_URL", &url)
            .env("BUNNY_STORAGE_URL", &url)
            .env("BUNNY_STREAM_KEY", "test-stream-key")
            .env("BUNNY_STORAGE_KEY", "test-storage-key");
    }

    cmd
}
