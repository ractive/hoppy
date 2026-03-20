---
title: "Iteration 11 — E2E Lifecycle Tests (Rust Live API)"
type: iteration
date: 2026-03-19
tags:
  - iteration
  - testing
  - e2e
  - live-api
  - lifecycle
status: completed
branch: iter-11/e2e-lifecycle-testbooks
---

# Iteration 11 — E2E Lifecycle Tests (Rust Live API)

**Goal:** Rust live API tests that exercise full resource lifecycles (create -> get -> list -> update -> delete) against the live bunny.net API, plus a `--record` flag for fixture capture.

## Architecture

Three test layers, sharing fixtures:

| Layer | Location | Purpose | Mocking |
|-------|----------|---------|---------|
| API unit tests | `crates/bunny-api-*/tests/` | HTTP client correctness, deserialization | wiremock |
| CLI E2E tests | `tests/cli_*.rs` | CLI arg parsing -> correct HTTP request, output formatting | wiremock |
| Live E2E tests | Same files, `#[cfg(feature = "live-api")]` | Full lifecycle against real API | None |

**Division of labour:**
- **Rust wiremock tests** (crate-level + CLI-level) -> fast CI tests with mock HTTP
- **Rust live tests** (`#[cfg(feature = "live-api")]`) -> lifecycle tests against real API
- **`--record` flag** -> captures API responses as fixture files for wiremock tests

## `--record=<dir>` Flag

Global CLI flag that records API response bodies to JSON files. Threads through `cli.rs -> main.rs -> commands/*.rs -> auth.rs -> *Client.with_record(path)`. Uses the same path as `--debug`.

```bash
hoppy --record=fixtures/ --format json pull-zone list
```

## Test Helpers (`tests/support/mod.rs`)

- `hoppy_live_json(args)` — runs `hoppy --format json <args>`, returns `LiveResult` with parsed JSON
- `hoppy_live_raw(args)` — runs `hoppy --yes <args>`, raw output
- `hoppy_live_json_yes(args)` — runs `hoppy --yes --format json <args>`
- `unique_name(prefix)` — generates `"prefix-{timestamp}-{counter}"` for unique resource names
- `run_lifecycle(|cleanup| { ... })` — panic-safe test wrapper with `CleanupStack`
- `CleanupStack` — collects delete commands, runs in reverse order even on panic

## Live Test Coverage (15 tests across 8 files)

| File | Test | Lifecycle steps |
|------|------|----------------|
| `cli_auth.rs` | `live_auth_check` | check -> assert billing info |
| `cli_pull_zone.rs` | `live_pull_zone_lifecycle` | create -> get -> list -> update -> verify -> purge -> delete |
| `cli_pull_zone.rs` | `live_pull_zone_get_nonexistent` | get 999999999 -> error |
| `cli_pull_zone.rs` | `live_pull_zone_update_nonexistent` | update 999999999 -> error |
| `cli_dns.rs` | `live_dns_zone_lifecycle` | create -> get -> list -> update -> verify -> delete |
| `cli_dns.rs` | `live_dns_record_lifecycle` | create zone -> add A -> list -> update -> delete -> delete zone |
| `cli_dns.rs` | `live_dns_record_mx_priority` | create zone -> add MX -> verify priority -> delete zone |
| `cli_storage_zone.rs` | `live_storage_zone_lifecycle` | create -> get -> list -> update -> verify -> delete |
| `cli_storage.rs` | `live_storage_file_ops` | create zone -> upload -> ls -> download+verify -> rm -> delete zone |
| `cli_stream.rs` | `live_stream_library_lifecycle` | create -> get -> list -> update -> verify -> delete |
| `cli_stream.rs` | `live_stream_collection_lifecycle` | create lib -> create coll -> get -> list -> update -> verify -> delete coll -> delete lib |
| `cli_script.rs` | `live_script_lifecycle` | create -> get -> list -> update -> code update -> code get -> publish -> releases -> delete |
| `cli_script.rs` | `live_script_variable_lifecycle` | create script -> add -> list -> update -> upsert -> delete var -> delete script |
| `cli_script.rs` | `live_script_secret_lifecycle` | create script -> add -> list -> update -> upsert -> delete secret -> delete script |
| `cli_shield.rs` | `live_shield_lifecycle` | create PZ -> create SZ -> get -> list -> update -> WAF CRUD -> rate-limit CRUD -> access-list CRUD -> bot-detection -> delete PZ |

## Key Design Decisions

1. **Single monolithic lifecycle function per resource** — uses `run_lifecycle()` with `catch_unwind` for panic-safe cleanup
2. **`CleanupStack`** — register delete commands early, run in reverse order even on panic
3. **Feature flag gating** — `#[cfg(feature = "live-api")]` keeps live tests out of default CI
4. **Same files as mock tests** — live tests coexist with wiremock tests in same `tests/cli_*.rs` files
5. **Unique names with timestamps** — avoids collisions between test runs
6. **Containers deferred** — cost/complexity concerns

## Running Tests

```bash
# Mock tests (default, fast CI)
cargo test

# Live API tests only
BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_

# Compile check without running
cargo test --features live-api --no-run
```

**Deliverable:** `BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_` runs 15 lifecycle tests against bunny.net. All 95 mock tests unchanged.

## Related
- [[development-roadmap]] — project roadmap
- [[iterations/iteration-10-e2e-test-harness]] — previous iteration (superseded)
- [[iterations/rust-e2e-rewrite-plan]] — detailed rewrite plan
- [[testing/test-plan-v0.1.0]] — overall test plan
- [[research/cli-e2e-testing-research]] — research behind testing approach
