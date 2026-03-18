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
- [ ] Progress bar for upload/download (stderr, only if TTY) — deferred to iter 7 polish
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
- [ ] Video upload with progress bar — deferred to iter 7 polish

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
- [ ] WAF profiles command (`shield waf profiles`) — API client implemented, CLI not wired yet

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
  - [ ] CLI commands for Magic Containers — not yet wired (API client ready, CLI integration pending)

**Deliverable:** Deploy and manage edge scripts. Magic Containers API client implemented; CLI commands pending.

---

## Iteration 7 — Polish & Distribution

**Goal:** Production-ready release.

- [ ] Config file support: `~/.config/hoppy/config.toml` for defaults
- [ ] Shell completion install helper: `hoppy completions install bash|zsh|fish`
- [ ] `--dry-run` for mutating operations (show what would happen)
- [ ] `--wait` for async operations
- [ ] Man page generation
- [ ] GitHub Actions release workflow: build + upload binaries on version tag
- [ ] Homebrew formula / cargo install support
- [ ] Comprehensive README with examples for each service
- [ ] `hoppy auth check` — validate API key and print account info

**Deliverable:** Tagged v0.1.0 release with binaries for linux (x86_64, aarch64), macOS (x86_64, aarch64), windows (x86_64).

---

## Possible Future Iterations

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
| 6 — Scripting + Containers | Two services, API client + CLI for scripting, API client for containers | Medium |
| 7 — Polish & Release | CI/CD, packaging, docs | Medium |

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
| 2026-03-18 | Magic Containers API client hand-written from docs | No OpenAPI spec available for Magic Containers. Client generated manually from https://docs.bunny.net documentation pages (47 endpoints). Uses camelCase serde (like Shield), cursor-based pagination, ProblemDetails+ErrorDetails error handling. CLI wiring deferred. |
| 2026-03-18 | `Deploy` renamed to `Publish` for edge scripts | The bunny.net API endpoint is `POST /compute/script/{id}/publish`, not "deploy". CLI command renamed to match. |
| 2026-03-18 | `deployment_key` excluded from JSON output | `#[serde(skip_serializing)]` on `EdgeScript.deployment_key` to prevent leaking deployment credentials. Same pattern as other crates. |
| 2026-03-18 | Compute API uses PascalCase like Core API | Confirmed via OpenAPI spec and fixture recording. All types use `#[serde(rename_all = "PascalCase")]`. |
| 2026-03-18 | Compute API `Items` can be null in paginated responses | Unlike Core API, Compute API may return `"Items": null` instead of `[]`. Custom `deserialize_null_as_empty_vec` handles this. PaginatedList intentionally kept separate from Core's version. |
