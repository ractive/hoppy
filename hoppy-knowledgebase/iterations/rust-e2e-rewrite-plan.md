---
date: 2026-03-19
status: completed
tags:
- testing
- e2e
- planning
title: "Plan: Rewrite Bun E2E Tests as Rust Tests"
type: plan
---

# Plan: Rewrite Bun E2E Tests as Rust Tests

## Architecture

Three test layers, sharing fixtures and mock setup:

| Layer | Location | Purpose | Mocking |
|-------|----------|---------|---------|
| API unit tests (existing) | `crates/bunny-api-*/tests/` | HTTP client sends correct requests, deserializes responses | wiremock |
| CLI E2E tests (new) | `tests/cli_*.rs` | CLI arg parsing → correct HTTP request, output formatting | wiremock |
| Live E2E tests (new, gated) | Same files, `#[cfg(feature = "live-api")]` | Smoke test against real API | None |

## Design Decisions

- **E2E tests live in `tests/` at repo root** — standard Rust integration test location
- **Shared helpers in `tests/support/mod.rs`** — not a separate crate, just a test module
- **Feature flag for live tests** — `cargo test --features live-api`, not env var detection
- **One mock server per test** — parallel-safe, no `#[serial]` needed for mock tests
- **Live tests use `#[serial]`** — they create real resources, must run sequentially
- **insta snapshots** on CLI stdout — verifies JSON and table output formatting
- **wiremock `expect(1)`** — verifies correct endpoint was called with correct method

## Dependencies to Add

Root `Cargo.toml`:

```toml
[features]
live-api = []

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
insta = { version = "1", features = ["json", "redactions"] }
wiremock = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

## Files to Create

### `tests/support/mod.rs` — Shared test helpers

```rust
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

/// Build a hoppy Command pointed at mock server(s).
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
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {relative_path}: {e}"))
}

#[cfg(feature = "live-api")]
pub fn hoppy_live_cmd() -> Command {
    let mut cmd = Command::cargo_bin("hoppy").expect("binary not found");
    assert!(std::env::var("BUNNY_API_KEY").is_ok(), "BUNNY_API_KEY required");
    cmd
}

#[cfg(feature = "live-api")]
pub fn unique_name(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}-{ts}-{n}")
}
```

### Test files — one per service

| File | Tests | Mock endpoints |
|------|-------|---------------|
| `tests/cli_auth.rs` | auth check (json, table) | GET /billing |
| `tests/cli_pull_zone.rs` | list, get, create, update, purge, delete, errors | /pullzone |
| `tests/cli_storage_zone.rs` | list, get, create, update, delete | /storagezone |
| `tests/cli_storage.rs` | upload, ls, download, rm | storage.bunnycdn.com |
| `tests/cli_dns.rs` | zone CRUD, record CRUD, MX priority | /dnszone |
| `tests/cli_stream.rs` | library CRUD, collection CRUD | video.bunnycdn.com |
| `tests/cli_script.rs` | script CRUD, code, publish, releases, variables, secrets | /compute |
| `tests/cli_shield.rs` | zone, WAF, rate-limit, access-list, bot-detection | /shield |

### Test patterns

**Mock test — verifies CLI sends correct request and formats output:**

```rust
#[tokio::test]
async fn pull_zone_list_json() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone"))
        .and(header("AccessKey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_raw(support::fixture("core/pullzone_list_paginated.json"), "application/json"))
        .expect(1)
        .mount(&server).await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "list"])
        .output().unwrap();

    assert!(output.status.success());
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}
```

**Error test — verifies CLI exits non-zero and shows error:**

```rust
#[tokio::test]
async fn pull_zone_get_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/pullzone/999999"))
        .respond_with(ResponseTemplate::new(404)
            .set_body_raw(support::fixture("core/error_not_found_storagezone.json"), "application/json"))
        .mount(&server).await;

    let output = support::hoppy_mock_cmd("test-api-key", &server.uri())
        .args(["--format", "json", "pull-zone", "get", "--id", "999999"])
        .output().unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("error"));
}
```

**Live lifecycle test — smoke test against real API:**

```rust
#[cfg(feature = "live-api")]
#[test]
fn pull_zone_lifecycle() {
    let name = support::unique_name("hoppy-e2e");

    // Create
    let out = support::hoppy_live_cmd()
        .args(["--format", "json", "pull-zone", "create",
               "--name", &name, "--origin-url", "https://example.com"])
        .output().unwrap();
    assert!(out.status.success());
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let id = json["Id"].to_string();

    // Get
    let out = support::hoppy_live_cmd()
        .args(["--format", "json", "pull-zone", "get", "--id", &id])
        .output().unwrap();
    assert!(out.status.success());

    // Delete (cleanup)
    let out = support::hoppy_live_cmd()
        .args(["--yes", "pull-zone", "delete", "--id", &id])
        .output().unwrap();
    assert!(out.status.success());
}
```

## What each test layer verifies

**Wiremock CLI tests verify:**
- Exit code 0 on success, non-zero on error
- Correct HTTP method + path hits the mock (via `expect(1)`)
- JSON output format (via insta snapshot)
- Table output format (via insta snapshot)
- `--yes` flag bypasses confirmation
- Error messages appear on stderr

**Wiremock CLI tests do NOT verify:**
- JSON response content (the API's job, not ours)
- Specific field values (fixtures are test data, not contract)

**Live E2E tests verify:**
- Create returns a valid ID
- Get/list finds the created resource
- Update modifies the resource
- Delete succeeds
- Exit code 0 throughout

## Env vars reference

| Env Var | Purpose |
|---------|---------|
| `BUNNY_API_KEY` | Auth for Core/Shield/Compute/Containers |
| `BUNNY_API_URL` | Override base URL (default: `https://api.bunny.net`) |
| `BUNNY_STORAGE_URL` | Override storage URL |
| `BUNNY_STORAGE_KEY` | Storage zone password |
| `BUNNY_STREAM_URL` | Override stream URL |
| `BUNNY_STREAM_KEY` | Stream API key |
| `BUNNY_CONTAINERS_URL` | Override containers URL |

For mock tests: all URL vars → mock server, `BUNNY_API_KEY` → `"test-api-key"`.

For stream/storage mock tests: must also set service-specific keys to prevent the CLI from trying to fetch them from the Core API.

## Implementation order

### Phase 1: Infrastructure
1. Add `[features]` and `[dev-dependencies]` to root `Cargo.toml`
2. Create `tests/support/mod.rs` with helpers

### Phase 2: First test (template)
3. Create `tests/cli_auth.rs` — simplest, one endpoint
4. Run `cargo test --test cli_auth` to validate
5. Run `cargo insta review` to accept snapshots

### Phase 3: Core services
6. `tests/cli_pull_zone.rs`
7. `tests/cli_storage_zone.rs`
8. `tests/cli_storage.rs`
9. `tests/cli_dns.rs`

### Phase 4: Additional services
10. `tests/cli_stream.rs`
11. `tests/cli_script.rs`
12. `tests/cli_shield.rs`

### Phase 5: Live tests
13. Add `#[cfg(feature = "live-api")]` modules to each file

### Phase 6: Cleanup
14. Delete `testbooks/` directory entirely
15. Update `hoppy-knowledgebase/adding-a-feature.md` (remove Bun references)
16. Update `hoppy-knowledgebase/development-roadmap.md` (remove Bun sections)

## Missing fixtures

Some CLI operations need fixtures not yet present:
- `core/pullzone_create.json` — can copy from `pullzone_get.json`
- Update responses can reuse existing get fixtures (API returns updated resource)
- Delete responses return 204 (no body needed)
- Purge returns 204 (no body needed)

## Related
- [[iterations/e2e-test-harness-plan]] — original test harness plan this builds on
- [[research/cli-e2e-testing-research]] — research behind the testing approach
- [[testing/test-plan-v0.1.0]] — overall test plan
- [[development-roadmap]] — iteration 11 in the roadmap
