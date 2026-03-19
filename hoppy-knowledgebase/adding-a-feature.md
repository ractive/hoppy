---
title: "Adding a New Feature"
date: 2026-03-19
tags:
  - development
  - checklist
  - testing
status: active
---

# Adding a New Feature

Step-by-step checklist for adding a new API feature to hoppy, from API client to tested CLI command.

## Checklist

### 1. API client

- [ ] Add request/response types to `crates/bunny-api-*/src/types.rs`
- [ ] Add client method to `crates/bunny-api-*/src/client.rs`
- [ ] Re-export new types from `crates/bunny-api-*/src/lib.rs`

### 2. CLI wiring

- [ ] Add subcommand variant to the relevant `src/commands/*.rs` handler
- [ ] Register in `src/cli.rs` (enum variant) and `src/main.rs` (dispatch)
- [ ] Implement JSON and table output formatting

### 3. Capture fixture from live API

Run the new command with `--debug` to capture the raw API response:

```bash
hoppy --debug --format json <your-new-command> 2>debug.log
```

The `<<< {...}` line in `debug.log` contains the raw API response body. Save it as a fixture:

```bash
grep '^<<<' debug.log | sed 's/^<<< //' | jq . > fixtures/<service>/<resource>_<action>.json
```

Alternatively, use the capture helper to do this automatically during a Bun E2E test run:

```bash
cd testbooks && CAPTURE_FIXTURES=1 bun test
```

### 4. Rust wiremock test

- [ ] Add `include_str!("../../../fixtures/<service>/<resource>_<action>.json")` constant
- [ ] Add wiremock test in `crates/bunny-api-*/tests/*_api.rs` using the fixture
- [ ] Cover both success and error paths (create error fixture manually if needed)

### 5. Bun E2E test book

- [ ] Add lifecycle steps to the appropriate `testbooks/*.test.ts` (or create a new test book)
- [ ] Follow the two-describe pattern: lifecycle (happy path) + error handling (unhappy path)
- [ ] Register cleanup with `onCleanupDelete()` in `afterAll`
- [ ] Update snapshots: `cd testbooks && bun test --update-snapshots`

### 6. Verify

```bash
cargo test --workspace
cd testbooks && bun test
```

## Fixture naming convention

Fixtures live in `fixtures/<service>/` and follow this pattern:

```
<resource>_<action>.json           # e.g. pullzone_get.json
<resource>_<action>_<modifier>.json # e.g. pullzone_list_paginated.json
error_<type>.json                  # e.g. error_unauthorized.json
error_<type>_<resource>.json       # e.g. error_not_found_dnszone.json
```

Service directories map to crate names:
- `core` → bunny-api-core (pull zones, storage zones, DNS, video libraries, billing)
- `storage` → bunny-api-storage (file operations)
- `stream` → bunny-api-stream (videos, collections)
- `shield` → bunny-api-shield (WAF, rate limiting, access lists)
- `compute` → bunny-api-compute (edge scripts, variables, secrets)
- `containers` → bunny-api-containers (Magic Containers)

## Shared fixtures: Rust and Bun

Both test suites use the same `fixtures/` directory as their source of truth for API response shapes:

- **Rust wiremock tests** use fixtures directly via `include_str!` as mock HTTP responses
- **Bun E2E tests** capture real API responses that can refresh these fixtures

When you capture a new fixture from a Bun test run, the same file is immediately available to Rust wiremock tests. This keeps both test suites in sync without manual copy-paste.
