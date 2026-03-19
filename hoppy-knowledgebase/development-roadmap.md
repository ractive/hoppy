---
title: "Hoppy Development Roadmap"
date: 2026-03-17
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

## Git Branching Strategy

**One branch per iteration**, merged to main via PR.

- Branch naming: `iter-0/skeleton`, `iter-0.5/codegen-experiment`, `iter-1/pull-zones`, etc.
- Main is always in a working state
- Each iteration gets a reviewable PR
- If an iteration goes sideways, we abandon the branch — main is safe
- No feature branches within iterations (each iteration is small enough to be atomic)

---

## Iteration 0 — Project Skeleton

**Goal:** Rust project compiles, CLI parses args, nothing talks to the network yet.

- [x] `cargo init` with workspace layout
- [x] Clap derive setup with nested subcommand structure (`hoppy <service> <action>`)
- [x] Global flags: `--format json|table|text`, `--debug`, `--quiet`, `--yes`, `--version`
- [x] `BUNNY_API_KEY` env var reading (validate presence, error if missing)
- [x] Output formatting module (json + table + text) with a dummy data struct
- [x] Error handling scaffold (anyhow, human-friendly error display, JSON errors when `--format json`)
- [x] Stderr for status/errors, stdout for data
- [x] `hoppy completions <shell>` subcommand (clap_complete)
- [x] Basic README with usage examples
- [x] CI: GitHub Actions workflow that builds and runs `hoppy --help` on linux/mac/windows

**Deliverable:** `hoppy --help` shows the command tree, `hoppy pull-zone list` prints "not implemented yet" cleanly.

---

## Iteration 0.5 — Codegen Experiment

**Goal:** Validate whether Progenitor can generate usable Rust clients from the bunny.net OpenAPI specs. Determine the codegen strategy before writing service implementations.

### OpenAPI Specs Inventory

All specs are **OpenAPI 3.0.x** (no Swagger 2.0):

| API | Spec URL | OAS Version | Endpoints | Schemas | Base URL |
|-----|----------|-------------|-----------|---------|----------|
| Core Platform | [public.json](https://core-api-public-docs.b-cdn.net/docs/v3/public.json) | 3.0.0 | ~65 | ~100 | `api.bunny.net` |
| Stream (Video) | [bunnynet-video-api.public.json](https://video.bunnycdn.com/openapi/bunnynet-video-api.public.json) | 3.0.0 | ~30 | ~50 | `video.bunnycdn.com` |
| Shield | [swagger.json](https://api.bunny.net/shield/docs/v1/swagger.json) | 3.0.4 | ~41 | ~60 | `api.bunny.net/shield` |
| Edge Scripting | [compute.json](https://core-api-public-docs.b-cdn.net/docs/v3/compute.json) | 3.0.0 | ~22 | ~24 | `api.bunny.net/compute` |
| Storage | [openapi.json](https://docs.bunny.net/api-reference/storage/openapi.json) | 3.0.0 | 4 | 2 | `{region}.storage.bunnycdn.com` |

### Experiment Plan

- [ ] Download all 5 OpenAPI spec files into `specs/` directory
- [ ] Install `cargo-progenitor`
- [ ] Run Progenitor against each spec, record results:
  - [ ] Core Platform — largest, most important
  - [ ] Stream — second priority
  - [ ] Shield — uses 3.0.4, may have quirks
  - [ ] Edge Scripting — smaller, good test case
  - [ ] Storage — tiny, baseline test
- [ ] For each spec, evaluate:
  - Does it generate without errors?
  - Do the generated types look correct?
  - Does the generated client compile?
  - Are the method signatures usable?
- [ ] If Progenitor fails on a spec, try minor spec fixes (remove unsupported features, fix schema issues)
- [ ] If Progenitor fails fundamentally, try `openapi-generator` as fallback
- [ ] Document results and decide strategy per API

### Expected Outcome

A decision matrix:

| API | Codegen? | Tool | Notes |
|-----|----------|------|-------|
| Core Platform | yes/no | progenitor / openapi-generator / hand-written | ... |
| Stream | yes/no | ... | ... |
| Shield | yes/no | ... | ... |
| Edge Scripting | yes/no | ... | ... |
| Storage | yes/no | ... | ... |

All specs get the same treatment — if codegen works, we use it for all 5, including Storage. Consistency over convenience.

### Integration Approach

If codegen works, the generated clients go into a workspace member crate:

```
hoppy/
  Cargo.toml          (workspace)
  crates/
    hoppy-cli/        (the CLI binary)
    bunny-api-core/   (Core Platform API client)
    bunny-api-stream/ (Stream API client)
    bunny-api-shield/ (Shield API client)
    bunny-api-compute/(Edge Scripting API client)
    bunny-api-storage/(Edge Storage API client)
  specs/              (downloaded OpenAPI JSON files)
```

The CLI crate depends on the generated crates and wraps their clients with our auth/output/error handling.

**Deliverable:** Decision document with codegen results per spec. Generated crates compile (or documented reasons why not).

---

## Iteration 1 — First Service: Pull Zones (Core API)

**Goal:** Full CRUD for pull zones against the live API. This proves out the entire vertical stack.

- [x] HTTP client setup (hand-written reqwest — codegen abandoned in iter 0.5)
- [x] Shared request/response plumbing: base URL, auth, error mapping
- [x] Debug logging of HTTP requests (`--debug` flag)
- [x] Pull Zone commands:
  - [x] `list` — paginated listing with `--search`, `--page`, `--per-page`
  - [x] `get --id <id>` — single pull zone details
  - [x] `create --name <name> --origin-url <url> [options]` — create pull zone
  - [x] `update --id <id> [options]` — update pull zone settings
  - [x] `delete --id <id> [--yes]` — delete with confirmation prompt
  - [x] `purge --id <id> [--cache-tag <tag>]` — purge cache (by tag or all)
- [x] Table output: pick sensible default columns (id, name, origin URL, status)
- [x] Pagination: `--page`, `--per-page` flags
- [ ] Integration test: at least one test that mocks the API response (deferred — need real API responses as fixtures first)

**Deliverable:** `BUNNY_API_KEY=xxx hoppy pull-zone list --format json` returns real data.

---

## Iteration 2 — Storage Zones + File Operations

**Goal:** Manage storage zones and upload/download files — this exercises the Storage API (different base URL, different auth key).

- [x] Storage Zone commands (Core API):
  - [x] `storage-zone list|get|create|update|delete`
- [x] Storage file commands (Storage API — different base URL):
  - [x] `storage upload --zone <name> --remote-path <path> --file <local-path>`
  - [x] `storage download --zone <name> --remote-path <path> [--output <local-path>]`
  - [x] `storage ls --zone <name> [--path <dir>]`
  - [x] `storage rm --zone <name> --remote-path <path> [--yes]`
- [x] Handle per-zone storage API key (from zone details or `BUNNY_STORAGE_KEY` env var)
- [x] Progress bar for upload/download (stderr, only if TTY) — done in iter 7
- [x] JSON list output should include pagination envelope (`current_page`, `total_items`, `has_more_items`), not just the items array — apply consistently across all list commands including pull zones
- [x] Integration tests with mock HTTP server (carried from iter 1 — record real API responses as fixtures first)
- [x] Consolidate duplicate `PaginatedList` and `ApiError` types across `bunny-api-core` and `bunny-api-compute` — investigated, intentionally kept separate with documentation (crates are independent, no shared dependency warranted)

**Deliverable:** Upload and download files to/from bunny.net storage.

---

## Iteration 3 — DNS

**Goal:** Manage DNS zones and records.

- [x] DNS zone commands:
  - [x] `dns zone list|get|create|update|delete`
  - [x] Zone update flags: custom nameservers, SOA email, logging, IP anonymization
  - [x] Pagination and search support
- [x] DNS record commands:
  - [x] `dns record list --zone-id <id>`
  - [x] `dns record add --zone-id <id> --type <A|AAAA|CNAME|...> --name <name> --value <value> [--ttl <seconds>]`
  - [x] `dns record update --zone-id <id> --record-id <id> [options]`
  - [x] `dns record delete --zone-id <id> --record-id <id> [--yes]`
  - [x] All record types: A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA + bunny-specific (Redirect, Flatten, PullZone, Script)
  - [x] Record add supports priority, weight, port, flags, tag, comment
- [x] Confirmation prompts for destructive operations (--yes to skip)
- [x] 15 wiremock integration tests with fixture-based responses
- [ ] Import/export zone files — deferred (API supports export as BIND file, import endpoint exists but not documented well enough for safe implementation)

**Deliverable:** Full DNS management via CLI.

---

## Iteration 4 — Stream (Video)

**Goal:** Manage video libraries and videos — exercises the Stream API (different base URL and API key).

- [x] Stream library commands:
  - [x] `stream library list|get|create|update|delete`
- [x] Stream video commands:
  - [x] `stream video list --library-id <id>`
  - [x] `stream video get --library-id <id> --video-id <id>`
  - [x] `stream video upload --library-id <id> --file <path>`
  - [x] `stream video delete --library-id <id> --video-id <id> [--yes]`
- [x] Handle stream API key (`BUNNY_STREAM_KEY` or derived from library)
- [x] Video upload with progress bar — done in iter 7

**Deliverable:** Upload and manage videos via CLI.

---

## Iteration 5 — Shield (Security)

**Goal:** Manage WAF, rate limiting, DDoS settings.

- [x] Shield zone commands:
  - [x] `shield zone list|get|get-by-pullzone|create|update`
- [x] Shield subcommands:
  - [x] `shield waf list-rules|get-rule|add-rule|update-rule|delete-rule`
  - [x] `shield rate-limit list|get|create|update|delete`
  - [x] `shield access-list list|get|create|update|delete|update-config`
  - [x] `shield bot-detection get|update`
- [x] DDoS configuration via shield zone update (--ddos-sensitivity, --ddos-execution-mode, --ddos-challenge-window)
- [x] Debug mode support (`--debug` flag)
- [x] Confirmation prompts for destructive operations (`--yes` to skip)
- [x] 27 wiremock integration tests with fixture-based responses
- [x] Error handling tests (401 unauthorized, 404 not found)
- [x] WAF profiles command (`shield waf profiles`) — wired in iter 7

**Deliverable:** Security configuration via CLI.

---

## Iteration 6 — Edge Scripting + Magic Containers

**Goal:** Manage serverless scripts and containers.

- [x] Edge scripting commands:
  - [x] `script list|get|create|update|delete` with full options (pagination, search, linked pull zones)
  - [x] `script publish` (replaces `deploy` — matches API endpoint name)
  - [x] `script code get|update` (update supports `--code` inline or `--file` path)
  - [x] `script release list|get-active` with pagination
  - [x] `script variable list|add|update|delete`
  - [x] `script secret list|add|update|delete`
  - [x] `script statistics` with `--date-from`, `--date-to`, `--hourly`
- [x] Debug mode support (`--debug` flag) added to ComputeClient
- [x] Confirmation prompts for destructive operations (`--yes` to skip)
- [x] `deployment_key` excluded from JSON output (`#[serde(skip_serializing)]`)
- [x] 28 wiremock integration tests with fixture-based responses
- [x] Error handling tests (401 unauthorized, 404 not found)
- [x] Request body validation in tests (body_json matchers)
- [x] Magic container API client (`bunny-api-containers` crate) — hand-written from docs (no OpenAPI spec available)
  - [x] 47 endpoints across 11 resource groups (applications, containers, registries, endpoints, volumes, autoscaling, regions, nodes, pods, limits, log forwarding)
  - [x] Full type coverage: all request/response structs, enums, cursor-based pagination
  - [x] Error handling via `ProblemDetails` + `ErrorDetails` (RFC 7807 pattern, like Shield)
  - [x] 13 unit tests (serde roundtrip, client construction, auth header)
  - [x] 57 wiremock integration tests with real API fixtures (all 47 endpoints + error handling + debug mode)
  - [x] Fix enum serde casing: API returns camelCase, added `rename_all = "camelCase"` to 10 enums
  - [x] CLI commands for Magic Containers — full `container` command tree wired (apps, templates, endpoints, volumes, registries, regions, nodes, pods, limits, log forwarding)

**Deliverable:** Deploy and manage edge scripts. Magic Containers API client and CLI commands fully implemented.

---

## Iteration 7 — Code Cleanup & Small Features

**Goal:** Quick wins — clean up deferred tech debt, add small features. No release infrastructure yet.

- [x] `hoppy auth check` — validate API key and print billing/account info (`GET /billing` endpoint + CLI command + 3 wiremock tests)
- [x] Replace `endpoint_suggestions: Vec<serde_json::Value>` in `bunny-api-containers` types with concrete `EndpointSuggestion` struct
- [x] Remove `CursorListJson` wrapper in CLI — serialize `CursorList<T>` directly in JSON mode (5 call sites updated)
- [x] Add `FromStr` impls to container enums (`RuntimeType`, `Granularity`, `RegistryType`, `LogForwardingType`, `SyslogFormat`) — replaced 5 hand-written `parse_*` helpers + 5 unit tests
- [x] Wire WAF profiles CLI command (`shield waf profiles`)
- [x] Wire Shield zone lookup by pull zone (`shield zone get-by-pullzone`) — already existed from iter 5
- [x] Wire container autoscaling commands (`container app autoscaling-get|autoscaling-update`)
- [x] Wire container region settings commands (`container app region-settings-get|region-settings-update`)
- [x] Wire container registry image commands (`container registry image-tags|image-digest|config-suggestions|search-public`)
- [x] Wire compute upsert commands (`script variable upsert`, `script secret upsert`)
- [x] Progress bars for storage upload and video upload (`indicatif` crate, streaming uploads with determinate bar, stderr only when TTY, suppressed by `--quiet`); storage download uses an indeterminate spinner (client buffers the full response before writing)
- [x] `bunny-api-containers` wiremock integration tests — already had 57 tests covering all endpoints (confirmed, no gaps)

**Deliverable:** Cleaner codebase, all deferred small items resolved.

---

## Iteration 8 — Release Readiness

**Goal:** Everything needed to ship v0.1.0 as a proper open-source release.

### Foundation
- [x] LICENSE file (MIT)
- [x] CHANGELOG.md for v0.1.0 (summarize all iterations)
- [x] Cargo.toml metadata: `repository`, `homepage`, `keywords`, `categories`, `readme`
- [x] Update CI workflow: trigger on push to main + PRs (not just `workflow_dispatch`), `--workspace` for clippy and test

### GitHub Actions Release Workflow
- [x] Trigger on tag push matching `v*` (e.g. `v0.1.0`)
- [x] Build matrix (6 targets):
  - `x86_64-unknown-linux-gnu` (ubuntu-latest, native)
  - `aarch64-unknown-linux-gnu` (ubuntu-latest, cross-rs)
  - `x86_64-apple-darwin` (macos-13, native)
  - `aarch64-apple-darwin` (macos-latest, native)
  - `x86_64-pc-windows-msvc` (windows-latest, native)
  - `aarch64-pc-windows-msvc` (windows-latest, native)
- [x] Package artifacts: `.tar.gz` (linux/macOS), `.zip` (Windows)
- [x] Each archive includes: binary, shell completions (bash/zsh/fish), man page, LICENSE, README
- [x] Generate `sha256sums.txt` for all archives
- [x] Create GitHub Release from tag, upload all archives + checksums
- [x] Pinned versions: cross@0.2.5, cargo-deb@3, cargo-generate-rpm@0.20 (all with --locked)

### Man Page Generation
- [x] Add `clap_mangen` dependency (xtask crate)
- [x] xtask generates 159 man pages from clap command tree
- [x] Bundle in release archives and packages

### Shell Completions
- [x] Keep stdout approach (`hoppy completions <shell>`) — industry standard
- [x] Bundle pre-generated completions in release archives
- [x] Include completions in deb/rpm/Homebrew packages (auto-installed to correct paths)
- [x] Document redirect commands in README

### Packaging
- [x] **Homebrew**: `ractive/homebrew-hoppy` tap repo created; formula auto-updated by release workflow
- [x] **cargo install**: `cargo install --git https://github.com/ractive/hoppy` documented in README
- [x] **deb**: `cargo-deb` with `[package.metadata.deb]` — completions + man pages as assets
- [x] **rpm**: `cargo-generate-rpm` with `[package.metadata.generate-rpm]` — same assets
- [ ] **winget**: Submit manifest to `microsoft/winget-pkgs` after first release

### README Overhaul
- [x] Installation section: Homebrew, cargo install --git, direct download, deb/rpm, build from source
- [x] Feature overview with service list
- [x] Usage examples organized by service
- [x] Shell completions with per-shell paths
- [x] Global options
- [x] Environment variables section
- [x] Badges: CI status, license
- [ ] Contributing section (skipped — not needed for v0.1.0)

### Not in scope for v0.1.0
- crates.io publishing (requires publishing all 6 sub-crates with proper versioning)
- Signed binaries / macOS notarization
- AUR / Nix / Scoop packages
- Auto-update mechanism
- `hoppy completions install` subcommand (stdout + package managers is sufficient)

**Deliverable:** Tagged v0.1.0 release with binaries for linux (x86_64, aarch64), macOS (x86_64, aarch64), windows (x86_64, aarch64). Installable via Homebrew, cargo install --git, direct download, deb, rpm, or winget.

---

## Iteration 9 — Gap Analysis & Missing CLI Commands

**Goal:** Audit all API client methods against wired CLI commands, wire any missing ones.

- [x] Stream video commands:
  - [x] `stream video update --library-id <id> --video-id <id> [--title <title>] [--collection-id <id>]`
  - [x] `stream video fetch --library-id <id> --url <url>` — ingest video from remote URL (async)
- [x] Stream collection commands:
  - [x] `stream collection list|get|create|update|delete --library-id <id>`
- [x] Edge scripting commands:
  - [x] `script rotate-deployment-key --id <id>` — rotate deployment key
- [x] URL query parameter redaction in `--debug` output (privacy)
- [x] v0.1.0 test plan document

**Deliverable:** All API client methods have corresponding CLI commands. No gaps between client crates and CLI layer.

---

## Iteration 10 — E2E Test Harness (Mock-Based)

**Goal:** CLI-level tests that invoke the actual `hoppy` binary against a wiremock server and assert on stdout/stderr/exit code.

- [x] `assert_cmd` + `wiremock` + `predicates` dev-dependencies
- [x] Test support module (`tests/e2e_support/`) with `cmd::hoppy()` builder and `server::start()`
- [x] Base URL env var overrides (`BUNNY_API_URL`, `BUNNY_CONTAINERS_URL`, `BUNNY_STREAM_URL`, `BUNNY_STORAGE_URL`) wired into all command handlers
- [x] 103 E2E tests across 10 test files (one per service + globals + auth)
- [x] `HOPPY_E2E_LIVE=1` flag scaffolded (all current tests skip in live mode)
- [x] `tools/e2e-report.sh` — runs tests, generates Obsidian-compatible markdown report with failure details
- [x] Persistent raw test output log (`hoppy-knowledgebase/e2e-test-output.log`)

**Deliverable:** `cargo test --test 'e2e_*'` runs 103 mock-based CLI tests. Obsidian report generated by `tools/e2e-report.sh`.

**Limitations identified:** All tests are isolated mock stubs — no lifecycle coverage (create→use→delete), no live API support despite the flag being scaffolded, no fixture recording. Removed in iteration 11 and replaced with Rust live API lifecycle tests.

---

## Iteration 11 — E2E Lifecycle Tests (Rust Live API)

**Goal:** Rust live API tests that exercise full resource lifecycles (create → get → list → update → delete) against the live bunny.net API, plus a `--record` flag for fixture capture.

### Architecture

Three test layers, sharing fixtures:

| Layer | Location | Purpose | Mocking |
|-------|----------|---------|---------|
| API unit tests | `crates/bunny-api-*/tests/` | HTTP client correctness, deserialization | wiremock |
| CLI E2E tests | `tests/cli_*.rs` | CLI arg parsing → correct HTTP request, output formatting | wiremock |
| Live E2E tests | Same files, `#[cfg(feature = "live-api")]` | Full lifecycle against real API | None |

**Division of labour:**
- **Rust wiremock tests** (crate-level + CLI-level) → fast CI tests with mock HTTP
- **Rust live tests** (`#[cfg(feature = "live-api")]`) → lifecycle tests against real API
- **`--record` flag** → captures API responses as fixture files for wiremock tests

### `--record=<dir>` Flag

Global CLI flag that records API response bodies to JSON files. Threads through `cli.rs → main.rs → commands/*.rs → auth.rs → *Client.with_record(path)`. Uses the same path as `--debug`.

```bash
hoppy --record=fixtures/ --format json pull-zone list
```

### Test Helpers (`tests/support/mod.rs`)

- `hoppy_live_json(args)` — runs `hoppy --format json <args>`, returns `LiveResult` with parsed JSON
- `hoppy_live_raw(args)` — runs `hoppy --yes <args>`, raw output
- `hoppy_live_json_yes(args)` — runs `hoppy --yes --format json <args>`
- `unique_name(prefix)` — generates `"prefix-{timestamp}-{counter}"` for unique resource names
- `run_lifecycle(|cleanup| { ... })` — panic-safe test wrapper with `CleanupStack`
- `CleanupStack` — collects delete commands, runs in reverse order even on panic

### Live Test Coverage (15 tests across 8 files)

| File | Test | Lifecycle steps |
|------|------|----------------|
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

### Running Tests

```bash
# Mock tests (default, fast CI)
cargo test

# Live API tests only
BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_

# Compile check without running
cargo test --features live-api --no-run
```

### Key Design Decisions

1. **Single monolithic lifecycle function per resource** — uses `run_lifecycle()` with `catch_unwind` for panic-safe cleanup
2. **`CleanupStack`** — register delete commands early, run in reverse order even on panic
3. **Feature flag gating** — `#[cfg(feature = "live-api")]` keeps live tests out of default CI
4. **Same files as mock tests** — live tests coexist with wiremock tests in same `tests/cli_*.rs` files
5. **Unique names with timestamps** — avoids collisions between test runs
6. **Containers deferred** — cost/complexity concerns

**Deliverable:** `BUNNY_API_KEY=xxx cargo test --features live-api -- --test-threads=1 live_` runs 15 lifecycle tests against bunny.net. All 95 mock tests unchanged.

---

## Possible Future Iterations

- **Config file support** — `~/.config/hoppy/config.toml` for defaults (API key, default format, etc.)
- **`--dry-run` for mutating operations** — show what would happen without executing
- **`--wait` for async operations** — poll until operation completes
- **DNS import/export** — BIND zone file import/export (API partially supports it)
- **Statistics/analytics commands** — `hoppy stats` for traffic, bandwidth, cache hit rates
- **Billing commands** — `hoppy billing summary`, invoices
- **Optimizer commands** — image transformation presets
- **AI image generation** — `hoppy ai generate --prompt "..."`
- **Database commands** — `hoppy db query --sql "SELECT ..."`
- **Bulk operations** — pipe JSON in, batch create/update/delete
- **Profile support** — multiple named API key profiles
- **Auto-update** — self-update mechanism
- **MCP server mode** — run as a Model Context Protocol server for direct LLM integration

---

## Iteration Sizing Estimate

| Iteration | Scope | Complexity |
|-----------|-------|------------|
| 0 — Skeleton | CLI framework, output, auth, CI | Small |
| 0.5 — Codegen Experiment | Test Progenitor on all specs | Small |
| 1 — Pull Zones | First full service, HTTP client | Medium |
| 2 — Storage | Second API, file I/O, progress bars | Medium |
| 3 — DNS | Straightforward CRUD + records | Small-Medium |
| 4 — Stream | Third API, video upload | Medium |
| 5 — Shield | Security features | Small-Medium |
| 6 — Scripting + Containers | Two services, API client + CLI for both scripting and containers | Medium-Large |
| 7 — Code Cleanup | Tech debt, small features, deferred items | Small |
| 8 — Release Readiness | CI/CD, packaging, docs, Homebrew | Medium |
| 9 — Gap Analysis | Wire missing CLI commands from audit | Small |
| 10 — E2E Test Harness | Mock-based CLI tests with wiremock | Medium |
| 11 — Lifecycle Test Books | Bun-based live API lifecycle tests with snapshots | Medium |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-17 | Codegen experiment before committing to approach | All 5 specs are OAS 3.0.x — codegen is viable and would prevent error-prone hand-written serde types. Experiment validates this. |
| 2026-03-17 | All specs get same codegen treatment | Consistency over convenience — if codegen works, use it for all 5 specs including Storage |
| 2026-03-17 | Pull Zones as first service | Most common bunny.net use case (CDN), exercises the core API, good proving ground |
| 2026-03-17 | No `--api-key` flag | clig.dev: don't pass secrets via flags (visible in ps/history). Use `BUNNY_API_KEY` env var only. |
| 2026-03-17 | Table output as default for TTY | Follows az/gcloud/clig.dev pattern. JSON auto-default when piped. |
| 2026-03-17 | Branch per iteration | Keeps main stable. Branch naming: `iter-N/description`. Merge via PR. |
| 2026-03-18 | Hand-written API clients confirmed | Codegen (Progenitor) abandoned in iter 0.5 — hand-written reqwest clients with serde types proved cleaner for bunny.net's PascalCase API |
| 2026-03-18 | `zone_security_key` excluded from JSON output | Security keys in API responses are deserialized but `#[serde(skip_serializing)]` prevents leaking them in CLI output |
| 2026-03-18 | Mock tests deferred until real API fixtures available | Synthetic mocks test assumptions, not reality — record real responses via `--debug` first |
| 2026-03-18 | wiremock for integration tests | Real API responses recorded as sanitized JSON fixtures, served by wiremock MockServer; client's `with_base_url` points at mock |
| 2026-03-18 | Always send pagination params on list endpoints | bunny.net API returns bare array without `page`/`perPage`, but paginated envelope with them — always send defaults to get consistent `PaginatedList` response |
| 2026-03-18 | Storage auth resolution: env var → Core API fallback | `BUNNY_STORAGE_KEY` checked first; if absent, fetch zone via Core API and use its `Password` field. Keeps simple case fast, complex case automatic. |
| 2026-03-18 | Streaming upload deferred | Attempted reqwest `Body::wrap_stream` but it added unnecessary deps to root crate. `tokio::fs::read` is simpler and sufficient until progress bar work in iter 7. |
| 2026-03-18 | PaginatedList/ApiError kept separate per crate | Intentionally duplicated across bunny-api-core and bunny-api-compute — crates are independent workspace members, shared extraction would add coupling without benefit. Documented in source. |
| 2026-03-18 | Storage zone list API rejects Accept header | bunny.net returns 401 if `Accept: application/json` header is sent on `/storagezone` endpoint — removed Accept header for this endpoint |
| 2026-03-18 | DNS record creation uses PUT | bunny.net uses `PUT /dnszone/{zoneId}/records` for record creation, not POST — unusual but documented in their OpenAPI spec |
| 2026-03-18 | DNS records embedded in zone response | No separate list-records endpoint — `GET /dnszone/{id}` returns the zone with all records in a `Records` array. `dns record list` fetches the full zone and extracts records. |
| 2026-03-18 | DNS zone import/export deferred | API supports `GET /dnszone/{id}/export` (BIND file) and `POST /dnszone/{zoneId}/import` but import endpoint lacks documentation detail. Deferred to avoid risky implementation. |
| 2026-03-18 | Stream API key resolution mirrors Storage pattern | `BUNNY_STREAM_KEY` env var checked first; if absent, fetch library via Core API and use its `ApiKey` field. Consistent with storage key resolution. |
| 2026-03-18 | Stream API PascalCase despite OpenAPI spec claiming camelCase | Live API returns PascalCase fields, matching Core API. OpenAPI spec is misleading. Using `#[serde(rename_all = "PascalCase")]` on all Stream types. |
| 2026-03-18 | Video upload progress bar deferred to iter 7 | Same as storage upload — `tokio::fs::read` loads entire file. Progress bar work deferred to polish iteration. |
| 2026-03-18 | VideoLibrary.ApiKey excluded from JSON output | `#[serde(skip_serializing)]` on `api_key` and `read_only_api_key` to prevent leaking credentials in CLI output. Same pattern as storage zone passwords. |
| 2026-03-18 | Stream API pagination has `ItemsPerPage` not `HasMoreItems` | Stream API uses `ItemsPerPage` field instead of `HasMoreItems`. CLI computes `has_more_items` from `current_page * items_per_page < total_items`. |
| 2026-03-18 | Shield API uses camelCase unlike Core API's PascalCase | Shield API uses `camelCase` for all JSON field names. All Shield types use `#[serde(rename_all = "camelCase")]`. |
| 2026-03-18 | DDoS has no dedicated CRUD — configured via Shield Zone update | DDoS sensitivity, execution mode, and challenge window are fields on the Shield Zone, not separate resources. CLI exposes them as `shield zone update` flags. |
| 2026-03-18 | Shield block-vpn/tor/datacentre are read-only on API responses | These fields appear in `ShieldZoneResponse` but cannot be set via the update endpoint's `ShieldZoneRequest`. CLI does not expose them as update flags. |
| 2026-03-18 | Shield enum values passed as integers on CLI | WAF action types, operator types, sensitivity levels etc. are passed as numeric values (matching the API's integer enum representation). `serde_json::from_value` converts to typed enums with descriptive error messages. |
| 2026-03-18 | Magic Containers API client hand-written from docs | No OpenAPI spec available for Magic Containers. Client generated manually from https://docs.bunny.net documentation pages (47 endpoints). Uses camelCase serde (like Shield), cursor-based pagination, ProblemDetails+ErrorDetails error handling. CLI fully wired: `container app`, `container template`, `container endpoint`, `container volume`, `container registry`, `container region`, `container node`, `container pod`, `container limits`, `container log-forwarding` sub-commands. |
| 2026-03-18 | `Deploy` renamed to `Publish` for edge scripts | The bunny.net API endpoint is `POST /compute/script/{id}/publish`, not "deploy". CLI command renamed to match. |
| 2026-03-18 | `deployment_key` excluded from JSON output | `#[serde(skip_serializing)]` on `EdgeScript.deployment_key` to prevent leaking deployment credentials. Same pattern as other crates. |
| 2026-03-18 | Compute API uses PascalCase like Core API | Confirmed via OpenAPI spec and fixture recording. All types use `#[serde(rename_all = "PascalCase")]`. |
| 2026-03-18 | Compute API `Items` can be null in paginated responses | Unlike Core API, Compute API may return `"Items": null` instead of `[]`. Custom `deserialize_null_as_empty_vec` handles this. PaginatedList intentionally kept separate from Core's version. |
| 2026-03-18 | Homebrew tap: `ractive/homebrew-hoppy` | User may have other formulas; single-formula tap naming (`homebrew-hoppy`) keeps things isolated |
| 2026-03-18 | winget manifest prepared but submitted after first release | winget-pkgs requires a review process; prepare manifest in repo, submit PR manually after v0.1.0 is published |
| 2026-03-18 | `cargo install --git` instead of crates.io for v0.1.0 | Publishing to crates.io requires all 7 crates (6 api + 1 cli) published in dependency order with proper versioning — too much coordination for initial release |
| 2026-03-18 | Shell completions: stdout-only, no `install` subcommand | Industry standard (starship, rustup, gh, ripgrep, fd, bat). Package managers handle installation. Adding `install` later is non-breaking if needed. |
| 2026-03-18 | Windows aarch64 included in release matrix | `aarch64-pc-windows-msvc` builds natively on `windows-latest` runner — zero extra effort |
| 2026-03-18 | cross-rs for Linux aarch64 only | Native runners for everything else. cross-rs is more reliable than cargo-zigbuild for aarch64 (reported segfault issues with zigbuild). |
| 2026-03-19 | Rust `#[test]` not suitable for lifecycle E2E tests | cargo test doesn't guarantee ordering, has no shared state between test functions, and puts everything in one pass/fail if you use a single function. Lifecycle tests need sequential steps with state (captured IDs) passed between them. |
| 2026-03-19 | Bun test runner for lifecycle tests, not bash | Bash is natural for CLI testing but not cross-platform (Windows). Bun provides `describe`/`it` ordering, `afterAll` cleanup, snapshot testing with property matchers, native JSON, and runs on all platforms. |
| 2026-03-19 | Snapshots replace custom fixture recording | Bun's `toMatchSnapshot()` with property matchers captures API response structure. Committed snapshots serve as both regression tests and API documentation. No separate recording mechanism needed. |
