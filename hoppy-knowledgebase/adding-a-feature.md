---
title: Adding a New Feature
date: 2026-03-19
tags:
  - development
  - checklist
  - testing
status: active
type: guide
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

Alternatively, use `--record` to capture fixtures automatically:

```bash
hoppy --record=fixtures/ --format json <your-new-command>
```

> **Warning:** Recorded responses may contain secrets (e.g. `Password`, `ApiKey` fields).
> Always review and redact sensitive fields before committing fixture files.

### 4. Rust wiremock test

- [ ] Add `include_str!("../../../fixtures/<service>/<resource>_<action>.json")` constant
- [ ] Add wiremock test in `crates/bunny-api-*/tests/*_api.rs` using the fixture
- [ ] Cover both success and error paths (create error fixture manually if needed)

### 5. Rust live API test

- [ ] Add a `#[cfg(feature = "live-api")]` lifecycle test to the appropriate `tests/cli_*.rs`
- [ ] Use `run_lifecycle(|cleanup| { ... })` for panic-safe cleanup
- [ ] Register cleanup early with `cleanup.push(&[...])`
- [ ] Follow the pattern: create → get → list → update → verify → delete

### 6. Verify

```bash
cargo test --workspace
# Live API tests (optional, requires BUNNY_API_KEY)
BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_
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

## Fixture capture with --record

The `--record=<dir>` flag records API response bodies to JSON files during any hoppy invocation. This is useful for capturing new fixtures:

```bash
hoppy --record=fixtures/ --format json pull-zone list
```

Recorded files use the naming pattern `{METHOD}_{sanitized_path}.json` and are immediately usable by Rust wiremock tests.

## Related
- [[api/bunny-api-client-patterns]] — API client implementation patterns
- [[api/bunny-api-quirks]] — API quirks to watch for
- [[testing/test-plan-v0.1.0]] — overall test plan
- [[development-roadmap]] — project roadmap
