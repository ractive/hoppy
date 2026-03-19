---
title: "Hoppy Development Roadmap"
date: 2026-03-20
tags:
  - roadmap
  - planning
  - iterations
status: active
---

# Hoppy Development Roadmap

## Guiding Principles

- **Vertical slices**: Each iteration delivers something runnable end-to-end
- **Start narrow, widen later**: Get one service working well before adding more
- **Foundation first**: Invest early in the scaffolding (CLI framework, output formatting, auth, error handling) so adding services later is mechanical
- **Test with real API calls**: Each iteration should be testable against the live bunny.net API
- **Adding a feature**: Follow the [[adding-a-feature]] checklist
- **Decisions**: See [[decision-log]] for architectural choices and rationale

## Git Branching Strategy

**One branch per iteration**, merged to main via PR.

- Branch naming: `iter-0/skeleton`, `iter-0.5/codegen-experiment`, `iter-1/pull-zones`, etc.
- Main is always in a working state
- Each iteration gets a reviewable PR
- If an iteration goes sideways, we abandon the branch — main is safe

---

## Completed Iterations

| Iter | Branch | Summary |
|------|--------|---------|
| 0 | `iter-0/skeleton` | Project skeleton: clap CLI, output formatting, auth, error handling, CI |
| 0.5 | `iter-0.5/codegen-experiment` | Evaluated Progenitor codegen → abandoned in favor of hand-written clients |
| 1 | `iter-1/pull-zones` | Full Pull Zone CRUD — first vertical slice proving the stack |
| 2 | `iter-2/storage` | Storage Zones + file operations (upload/download/ls/rm), per-zone auth |
| 3 | `iter-3/dns` | DNS zones + records (all record types), 15 wiremock tests |
| 4 | `iter-4/stream` | Stream libraries + videos + collections, Stream API key resolution |
| 5 | `iter-5/shield` | Shield zones, WAF, rate limiting, access lists, bot detection, 27 wiremock tests |
| 6 | `iter-6/scripting-containers` | Edge scripting (scripts, variables, secrets, releases) + Magic Containers (47 endpoints, 57 wiremock tests) |
| 7 | `iter-7/cleanup` | Auth check, FromStr impls, progress bars, WAF profiles, deferred items |
| 8 | `iter-8/release` | Release workflow (6 targets), Homebrew, deb/rpm, man pages, shell completions, README |
| 9 | `iter-9/gap-analysis` | Wired missing CLI commands (stream video update/fetch, collections, script rotate-key), query param redaction |
| 10 | `iter-10/e2e-test-harness` | 103 mock-based CLI E2E tests with assert_cmd + wiremock. **Superseded by iter 11.** |
| 11 | `iter-11/e2e-lifecycle-testbooks` | Replaced iter 10 harness with wiremock + insta snapshot tests and `--features live-api` lifecycle tests |

---

## Current State (post iter 11)

### Test Architecture

| Layer | Location | Purpose | Mocking |
|-------|----------|---------|---------|
| API unit tests | `crates/bunny-api-*/tests/` | HTTP client correctness, deserialization | wiremock |
| CLI E2E tests | `tests/cli_*.rs` | CLI arg parsing → correct HTTP request, output formatting | wiremock + insta |
| Live E2E tests | Same files, `#[cfg(feature = "live-api")]` | Full lifecycle against real API | None |

### Running Tests

```bash
# Mock tests (default, fast CI)
cargo test

# Live API tests only
BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_

# Compile check without running
cargo test --features live-api --no-run
```

### Live Test Coverage (15 tests across 8 files)

| File | Test | Lifecycle |
|------|------|-----------|
| `cli_auth.rs` | `live_auth_check` | check → assert billing info |
| `cli_pull_zone.rs` | `live_pull_zone_lifecycle` | create → get → list → update → verify → purge → delete |
| `cli_pull_zone.rs` | `live_pull_zone_get_nonexistent` | get 999999999 → error |
| `cli_pull_zone.rs` | `live_pull_zone_update_nonexistent` | update 999999999 → error |
| `cli_dns.rs` | `live_dns_zone_lifecycle` | create → get → list → update → verify → delete |
| `cli_dns.rs` | `live_dns_record_lifecycle` | create zone → add A → list → update → delete → delete zone |
| `cli_dns.rs` | `live_dns_record_mx_priority` | create zone → add MX → verify priority → delete zone |
| `cli_storage_zone.rs` | `live_storage_zone_lifecycle` | create → get → list → update → verify → delete |
| `cli_storage.rs` | `live_storage_file_ops` | create zone → upload → ls → download+verify → rm → delete zone |
| `cli_stream.rs` | `live_stream_library_lifecycle` | create → get → list → update → verify → delete |
| `cli_stream.rs` | `live_stream_collection_lifecycle` | create lib → create coll → get → list → update → verify → delete coll → delete lib |
| `cli_script.rs` | `live_script_lifecycle` | create → get → list → update → code update → code get → publish → releases → delete |
| `cli_script.rs` | `live_script_variable_lifecycle` | create script → add → list → update → upsert → delete var → delete script |
| `cli_script.rs` | `live_script_secret_lifecycle` | create script → add → list → update → upsert → delete secret → delete script |
| `cli_shield.rs` | `live_shield_lifecycle` | create PZ → create SZ → get → list → update → WAF CRUD → rate-limit CRUD → access-list CRUD → bot-detection → delete PZ |

---

## Not Yet Done

- [ ] winget: Submit manifest to `microsoft/winget-pkgs` after first release
- [ ] DNS import/export — deferred (API partially supports it)
- [ ] Containers live E2E tests — deferred (cost/complexity)

---

## Possible Future Iterations

- **Config file support** — `~/.config/hoppy/config.toml` for defaults
- **`--dry-run` for mutating operations**
- **`--wait` for async operations** — poll until complete
- **DNS import/export** — BIND zone file import/export
- **Statistics/analytics commands**
- **Billing commands**
- **Bulk operations** — pipe JSON in, batch create/update/delete
- **Profile support** — multiple named API key profiles
- **Auto-update** — self-update mechanism
- **MCP server mode** — run as a Model Context Protocol server
