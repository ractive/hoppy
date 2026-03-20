---
title: "E2E Test Harness Plan"
date: 2026-03-19
tags:
  - testing
  - e2e
  - cli
  - planning
status: approved
---

# E2E Test Harness for Hoppy CLI

## Context

Hoppy has 193 wiremock integration tests at the API client level and 95 JSON fixtures, but **zero CLI-level tests**. We need an E2E harness that invokes the actual `hoppy` binary, points it at a mock server, and asserts on stdout/stderr/exit code. The harness should also support running against the real bunny.net API and generate an Obsidian-compatible markdown test report.

## Research Summary

Evaluated multiple approaches (see [[research/cli-e2e-testing-research]] for details):

| Approach | Verdict |
|----------|---------|
| `assert_cmd` + wiremock (Rust) | **Selected** — type-safe, reuses existing fixtures, runs via `cargo test` |
| Shell scripts (bats-core) | Rejected — brittle string matching, no fixture reuse |
| Python pytest | Rejected — adds Python dependency to Rust project |
| `trycmd` (snapshot testing) | Complementary — good for help text but can't handle API interaction |
| `httpmock` (record/replay) | Deferred — wiremock-rs doesn't support record/replay but `httpmock` does; could add later for recording new fixtures |

## Architecture

```
┌──────────────────┐     env vars      ┌──────────────┐
│  assert_cmd      │ ───────────────── │  hoppy binary │
│  (test runner)   │  BUNNY_API_URL    │  (built by    │
│                  │  BUNNY_API_KEY    │   cargo)      │
│  starts wiremock │  etc.             │              │
│  mounts fixtures │                   │  reads env    │
│  asserts stdout  │                   │  calls API    │
└────────┬─────────┘                   └──────┬───────┘
         │                                     │
         │ configures                          │ HTTP
         ▼                                     ▼
┌──────────────────┐                   ┌──────────────┐
│  wiremock server │ ◄──── requests ── │  API clients  │
│  (per test)      │                   │  (reqwest)    │
│  returns fixtures│                   │              │
└──────────────────┘                   └──────────────┘
```

## Implementation Steps

### Step 1: Add dev-dependencies to root `Cargo.toml`

```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
wiremock = "0.6"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

### Step 2: Add base URL env var overrides

The CLI currently hardcodes base URLs via `Client::new()`. Each client already has `with_base_url()`, but the CLI never uses it. Add env var overrides:

**`src/auth.rs`** — Add 4 helper functions:
- `get_api_url() -> Option<String>` — reads `BUNNY_API_URL`
- `get_containers_url() -> Option<String>` — reads `BUNNY_CONTAINERS_URL`
- `get_stream_url() -> Option<String>` — reads `BUNNY_STREAM_URL`
- `get_storage_url() -> Option<String>` — reads `BUNNY_STORAGE_URL`

**Command handler files** — Wire up the overrides:

| File | Client | Env var | Call sites |
|------|--------|---------|------------|
| `src/commands/pull_zone.rs` | CoreClient | `BUNNY_API_URL` | 1 |
| `src/commands/storage_zone.rs` | CoreClient | `BUNNY_API_URL` | 1 |
| `src/commands/dns.rs` | CoreClient | `BUNNY_API_URL` | 1 |
| `src/commands/auth.rs` | CoreClient | `BUNNY_API_URL` | 1 |
| `src/commands/shield.rs` | ShieldClient | `BUNNY_API_URL` | 5 |
| `src/commands/script.rs` | ComputeClient | `BUNNY_API_URL` | 1 |
| `src/commands/container.rs` | ContainersClient | `BUNNY_CONTAINERS_URL` | 1 |
| `src/commands/stream.rs` | CoreClient + StreamClient | `BUNNY_API_URL` + `BUNNY_STREAM_URL` | 3 |
| `src/commands/storage.rs` | CoreClient + StorageClient | `BUNNY_API_URL` + `BUNNY_STORAGE_URL` | 2 |

Pattern:
```rust
let client = if let Some(url) = auth::get_api_url() {
    CoreClient::with_base_url(auth::get_api_key()?, url).with_debug(debug)
} else {
    CoreClient::new(auth::get_api_key()?).with_debug(debug)
};
```

### Step 3: Test support module

```
tests/
  e2e_support/
    mod.rs          — re-exports
    server.rs       — start() -> MockServer
    cmd.rs          — hoppy(&MockServer) -> Command (sets all env vars)
```

`cmd::hoppy()` sets `BUNNY_API_KEY=test-api-key` and all `*_URL` env vars to point at the mock server.

### Step 4: Fixture symlinks

```
tests/fixtures/
  core/        -> ../../crates/bunny-api-core/tests/fixtures
  compute/     -> ../../crates/bunny-api-compute/tests/fixtures
  containers/  -> ../../crates/bunny-api-containers/tests/fixtures
  shield/      -> ../../crates/bunny-api-shield/tests/fixtures
  storage/     -> ../../crates/bunny-api-storage/tests/fixtures
  stream/      -> ../../crates/bunny-api-stream/tests/fixtures
```

### Step 5: E2E test files (one per command)

| File | Tests |
|------|-------|
| `tests/e2e_global.rs` | help text, version, missing API key, unknown commands |
| `tests/e2e_auth.rs` | auth check |
| `tests/e2e_pull_zone.rs` | list (table + json), get, create, update, delete, purge |
| `tests/e2e_storage_zone.rs` | list, get, create, update, delete |
| `tests/e2e_dns.rs` | zone CRUD, record CRUD |
| `tests/e2e_stream.rs` | library CRUD, video list/get/update/fetch/delete, collection CRUD |
| `tests/e2e_shield.rs` | zone, WAF, rate-limit, access-list, bot-detection |
| `tests/e2e_script.rs` | CRUD, publish, code, release, variable, secret, rotate-key |
| `tests/e2e_container.rs` | app, template, endpoint, volume, registry, etc. |
| `tests/e2e_storage.rs` | ls, upload, download, rm |

**Start with `e2e_global.rs` + `e2e_pull_zone.rs` + `e2e_auth.rs` as templates, then expand.**

### Step 6: Report generator script

**`tools/e2e-report.sh`** — Runs `cargo test --test 'e2e_*'` with `--no-fail-fast` so all results are collected even when some tests fail. Captures plain-text test output, parses `test ... ok` and `test ... FAILED` lines with bash regex, and writes `hoppy-knowledgebase/e2e-test-report.md` with Obsidian-compatible YAML frontmatter and a flat (ungrouped) checkbox list. The report file is gitignored.

Example output:
```markdown
- [x] `pull_zone_list_table_output`
- [x] `pull_zone_list_json_output`
- [ ] `pull_zone_get_not_found` **FAILED**
```

### Step 7: Dual mode support

- **Mock mode** (default): `cargo test --test 'e2e_*'` — tests start wiremock, no network
- **Live mode**: `HOPPY_E2E_LIVE=1 BUNNY_API_KEY=real-key cargo test --test 'e2e_*'` — tests skip mock setup, use real API
- **Recording**: Deferred — can add `httpmock` forwarding mode later to record real API responses as new fixtures

## Key Design Decisions

1. **One MockServer per test** — full isolation, parallel-safe, matches existing crate test pattern
2. **Symlink fixtures** — no duplication, new crate fixtures automatically available to E2E tests
3. **`BUNNY_STREAM_KEY`/`BUNNY_STORAGE_KEY` bypass key resolution** — avoids needing to mock the CoreClient fallback in stream/storage tests
4. **Shell script for report** — minimal tooling, Obsidian-compatible output
5. **Start small** — implement global + pull-zone + auth first, then expand incrementally
6. **Single mock server per test** — wiremock matches on full paths, so CoreClient (`/pullzone`), ComputeClient (`/compute/script`), and ShieldClient (`/shield/...`) can all share one server

## Adding New Tests Later

1. **New CLI command**: Add a test function to the appropriate `e2e_*.rs` file
2. **New fixture needed**: Add JSON file to the relevant crate's `tests/fixtures/` — symlinks make it automatically available
3. **New service/crate**: Create a new `tests/e2e_newservice.rs`, add fixture symlink
4. **Recording new fixture from live API**: Run command with `--debug`, capture JSON response, save to fixtures directory

## Verification

1. `cargo test --test e2e_global` — help/version tests pass
2. `cargo test --test e2e_pull_zone` — mock server tests pass
3. `bash tools/e2e-report.sh` — generates markdown report
4. Open report in Obsidian — checkboxes render correctly

## Related
- [[research/cli-e2e-testing-research]] — research that informed this plan
- [[iterations/rust-e2e-rewrite-plan]] — subsequent rewrite plan
- [[testing/test-plan-v0.1.0]] — overall test plan
- [[development-roadmap]] — iteration 10 in the roadmap
