---
date: 2026-03-20
status: active
tags:
- architecture
- decision
title: Decision Log
type: log
---

# Decision Log

Significant architectural and design decisions made during development. API-specific quirks are documented separately in [[api/bunny-net-api-quirks]].

## Architecture

| Decision | Rationale |
|----------|-----------|
| Hand-written API clients (not codegen) | Progenitor codegen produced 51K lines vs 4K hand-written. PascalCase API made codegen awkward. See [[research/hand-written-experiment-results]] |
| No `--api-key` flag | clig.dev: don't pass secrets via flags (visible in ps/history). `BUNNY_API_KEY` env var only |
| Table output as default for TTY | Follows az/gcloud/clig.dev pattern. JSON auto-default when piped |
| Branch per iteration | Keeps main stable. Branch naming: `iter-N/description`. Merge via PR |
| PaginatedList/ApiError kept separate per crate | Intentionally duplicated across bunny-net-api-core and bunny-net-api-compute — independent workspace members, shared extraction would add coupling without benefit |
| Consolidate bunny-net-api-* into one bunny-net-api crate (iter-32) | 8 per-service crates were premature splitting. No downstream consumer exists (nothing published), every release bumped all 8 in lockstep, cross-cutting changes multiplied per crate. One `bunny-net-api` lib with feature-gated modules mirrors hyalo's shape. CLI moves to `hoppy-cli` package (binary still `hoppy`). Install: `cargo install hoppy-cli`. |
| Magic Containers API hand-written from docs | No OpenAPI spec available. 47 endpoints. Uses camelCase serde (like Shield), cursor-based pagination, ProblemDetails+ErrorDetails error handling |

## Security

| Decision | Rationale |
|----------|-----------|
| `zone_security_key` excluded from JSON output | `#[serde(skip_serializing)]` prevents leaking security keys in CLI output |
| `VideoLibrary.ApiKey` excluded from JSON output | Same pattern — `api_key` and `read_only_api_key` skipped |
| `deployment_key` excluded from JSON output | Same pattern for EdgeScript credentials |
| Cross-cutting `--reveal` redaction layer (iter-21) | Magic-Container env values, storage-zone passwords, DB tokens are masked by default in JSON, table, and text output. Operators opt in with `--reveal` (all secrets) or `--reveal-env KEY` (one env var). Replaces ad-hoc per-field skipping for fields that operators legitimately need to read |
| Redaction defaults to ON even with `--format json` (iter-21) | A `--format json \| jq` pipeline must not silently leak a secret into a logfile. No env var auto-opts-out — only the explicit `--reveal` flag does |
| Destructive `template env` requires typed phrase (iter-21) | `--clear` and a shrinking `--replace-all` need the operator to type "wipe" / "replace". `--yes` alone is not sufficient — too easy to fat-finger after a successful prior `--yes` invocation |
| `app delete` refuses by default if auto-PZs exist (iter-21) | Magic-Container CDN endpoints create auto-managed Pull Zones that the app DELETE doesn't cascade to. Operators must explicitly choose `--cascade` (delete both) or `--no-cascade` (delete app, print orphan IDs). No silent billable orphan |
| `app create` returns full document by default (iter-21) | Provisioning a working stack used to take 3+ `app get` round-trips to chain template / endpoint ids. Default now returns the full app; `--minimal` opts back into the legacy `{"id": "..."}` shape |

## Auth Resolution

| Decision | Rationale |
|----------|-----------|
| Storage: env var → Core API fallback | `BUNNY_STORAGE_KEY` checked first; if absent, fetch zone via Core API and use its `Password` field |
| Stream: mirrors Storage pattern | `BUNNY_STREAM_KEY` checked first; if absent, fetch library via Core API and use its `ApiKey` field |
| Always send pagination params on list endpoints | bunny.net returns bare array without `page`/`perPage`, but paginated envelope with them |

## Testing

| Decision | Rationale |
|----------|-----------|
| wiremock for integration tests | Real API responses recorded as sanitized JSON fixtures, served by wiremock MockServer |
| Three test layers | API unit (wiremock), CLI E2E (wiremock + insta), Live lifecycle (`--features live-api`) |
| `--record=<dir>` flag for fixture capture | Threads through CLI → commands → clients. Best-effort, dev-only |
| `run_lifecycle()` with `CleanupStack` | Panic-safe cleanup via `catch_unwind`, delete commands run in reverse order |

## Release & Packaging

| Decision | Rationale |
|----------|-----------|
| `cargo install --git` instead of crates.io | Publishing to crates.io requires all 7 crates published in dependency order — too much for v0.1.0 |
| Homebrew tap: `ractive/homebrew-tap` | Use shared `ractive/homebrew-tap` (multi-formula) — `brew tap ractive/tap && brew install hoppy`. Supersedes earlier 2026-03-18 decision to use a dedicated `ractive/homebrew-hoppy` tap. |
| Shell completions: stdout-only | Industry standard (starship, rustup, gh). Package managers handle installation |
| cross-rs for Linux aarch64 only | Native runners for everything else. cross-rs more reliable than cargo-zigbuild |
| Windows aarch64 included | Builds natively on `windows-latest` runner — zero extra effort |

## API-Specific Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-18 | Storage zone list API rejects Accept header | bunny.net returns 401 if `Accept: application/json` header is sent on `/storagezone` — removed Accept header |
| 2026-03-18 | DNS record creation uses PUT | bunny.net uses `PUT /dnszone/{zoneId}/records` for creation, not POST — unusual but documented in OpenAPI spec |
| 2026-03-18 | DNS records embedded in zone response | No separate list-records endpoint — `GET /dnszone/{id}` returns the zone with all records in a `Records` array |
| 2026-03-18 | Stream API PascalCase despite OpenAPI spec claiming camelCase | Live API returns PascalCase fields. OpenAPI spec is misleading. Using `#[serde(rename_all = "PascalCase")]` |
| 2026-03-18 | Stream API pagination has `ItemsPerPage` not `HasMoreItems` | CLI computes `has_more_items` from `current_page * items_per_page < total_items` |
| 2026-03-18 | Shield API uses camelCase unlike Core API's PascalCase | All Shield types use `#[serde(rename_all = "camelCase")]` |
| 2026-03-18 | DDoS has no dedicated CRUD — configured via Shield Zone update | DDoS sensitivity, execution mode, challenge window are fields on the Shield Zone, not separate resources |
| 2026-03-18 | Shield block-vpn/tor/datacentre are read-only on API | Appear in response but cannot be set via update endpoint. CLI does not expose as update flags |
| 2026-03-18 | Shield enum values passed as integers on CLI | Matches API's integer enum representation. `serde_json::from_value` converts to typed enums |
| 2026-03-18 | `Deploy` renamed to `Publish` for edge scripts | API endpoint is `POST /compute/script/{id}/publish`, not "deploy" — CLI matches the API |
| 2026-03-18 | Compute API uses PascalCase like Core API | Confirmed via OpenAPI spec and fixture recording |
| 2026-03-18 | Compute API `Items` can be null in paginated responses | Custom `deserialize_null_as_empty_vec` handles `"Items": null` instead of `[]` |

## Chronological Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-17 | Codegen experiment before committing to approach | All 5 specs are OAS 3.0.x — codegen viable, experiment validates |
| 2026-03-17 | All specs get same codegen treatment | Consistency over convenience |
| 2026-03-17 | Pull Zones as first service | Most common bunny.net use case (CDN), exercises the core API |
| 2026-03-17 | No `--api-key` flag | clig.dev: don't pass secrets via flags (visible in ps/history) |
| 2026-03-17 | Table output as default for TTY | Follows az/gcloud/clig.dev pattern |
| 2026-03-17 | Branch per iteration | Keeps main stable |
| 2026-03-18 | Hand-written API clients confirmed | Progenitor abandoned in iter 0.5 — hand-written proved cleaner for PascalCase API |
| 2026-03-18 | Sensitive fields excluded from JSON output | `#[serde(skip_serializing)]` on `zone_security_key`, `api_key`, `deployment_key` |
| 2026-03-18 | Mock tests deferred until real API fixtures available | Synthetic mocks test assumptions, not reality |
| 2026-03-18 | wiremock for integration tests | Real API responses as sanitized JSON fixtures |
| 2026-03-18 | Always send pagination params on list endpoints | API returns bare array without params, paginated envelope with them |
| 2026-03-18 | Storage auth: env var → Core API fallback | `BUNNY_STORAGE_KEY` first; if absent, fetch zone and use `Password` field |
| 2026-03-18 | Streaming upload deferred | `tokio::fs::read` simpler and sufficient until progress bar work in iter 7 |
| 2026-03-18 | PaginatedList/ApiError kept separate per crate | Independent workspace members, shared extraction would add coupling without benefit |
| 2026-03-18 | Stream auth mirrors Storage pattern | `BUNNY_STREAM_KEY` first; if absent, fetch library and use `ApiKey` field |
| 2026-03-18 | DNS import/export deferred | Import endpoint lacks documentation detail |
| 2026-03-18 | Magic Containers hand-written from docs | No OpenAPI spec available. 47 endpoints, camelCase serde, cursor-based pagination |
| 2026-03-18 | Homebrew tap: `ractive/homebrew-hoppy` | Single-formula tap keeps things isolated |
| 2026-03-18 | winget manifest submitted after first release | winget-pkgs requires review process |
| 2026-03-18 | `cargo install --git` instead of crates.io | Publishing all 7 crates in order too much for initial release |
| 2026-03-18 | Shell completions: stdout-only | Industry standard. Package managers handle installation |
| 2026-03-18 | Windows aarch64 included | Builds natively on `windows-latest` — zero extra effort |
| 2026-03-18 | cross-rs for Linux aarch64 only | More reliable than cargo-zigbuild |
| 2026-03-19 | Feature flag for live tests | `cargo test --features live-api`, not env var detection |
| 2026-03-19 | `run_lifecycle()` with `CleanupStack` | Panic-safe cleanup via `catch_unwind`, delete commands in reverse order |
| 2026-05-07 | Forward-compat repr-enum deserialisation | Lossy `deserialize_repr_option` over an `Unknown(i32)` variant — keeps the change surface small (one helper per `Option<EnumType>`); round-trip drops unknown values, acceptable tradeoff vs full-response deserialisation failure (iter-19) |
| 2026-05-07 | Storage Zone passwords surfaced through redaction layer | `Password`/`ReadOnlyPassword` are no longer `skip_serializing`; CLI redacts via `redact_secrets_in_json` with `--reveal` opt-in. Operators were forced to `curl` for credentials without this (iter-19) |
| 2026-05-07 | `pull-zone create` ArgGroup for origin | Exactly-one-of `--origin-url` / `--storage-zone-id` enforced at clap parse time, no runtime API 400 (iter-19) |
| 2026-05-07 | Build provenance in `--version` | `build.rs` embeds short git SHA + bunny-spec mtime; long-version output for bug reports (iter-19) |
| 2026-05-07 | Single `e2e` test binary per crate | Cargo treats every top-level `.rs` under `tests/` as its own binary, each re-linking the crate-under-test from scratch. Collapsing into `tests/e2e/mod.rs` declared via `[[test]] name = "e2e"` cuts the linker pass to once per crate. Add new test files as `mod` declarations inside `tests/e2e/mod.rs` — never as new top-level files (iter-22) |
| 2026-05-09 | `bunny-net-api-` crate prefix kept (not `hoppy-` prefix) | Internal crates are split by **bunny.net API surface they wrap** (compute, containers, core, database, recording, shield, storage, stream), not by hoppy's own internal architecture. The `bunny-net-api-` prefix advertises what the crate is — a thin Rust client for one bunny.net surface — and lets each crate be reused outside hoppy if ever needed. Sibling project hyalo uses `hyalo-<domain>` because every crate is hyalo-internal; that doesn't apply here (iter-23) |
| 2026-05-09 | CLI verb conventions | **Top-level resources** use `create`/`delete`. **Items inside a parent collection** use `add`/`remove`. All collections expose `list` (and `get` where applicable). Mutation is always `update` — never `edit`/`set`/`modify`. Destructive ops accept `-y`/`--yes`; where `--dry-run` makes sense it's surfaced. Examples: `pull-zone create`, `dns zone create`, `dns record add`, `pull-zone hostname add`, `script variable add`. Documented exceptions: `storage ls`/`rm` (POSIX shorthand for filesystem-like ops), `shield waf add-rule`/`delete-rule` (matches flat API shape) (iter-23) |
| 2026-05-09 | Pedantic clippy deferred, not skipped | Hyalo enables `clippy::pedantic = warn` with a documented allow-list. Hoppy doesn't yet — turning it on at iter-23 would flood the build. The path: enable it in a follow-up iteration, fix or explicitly allow each surfaced lint with a one-line justification. Don't blanket-allow. Workspace lints currently set `unsafe_code = "forbid"` because no unsafe exists in tree (iter-23) |
| 2026-05-09 | Workspace dependency hoisting + `resolver = "3"` | All shared external deps live in `[workspace.dependencies]`; per-crate `Cargo.toml` files declare `clap.workspace = true` style. Cuts version drift and centralises bumps. `resolver = "3"` is the edition-2024 default; `[profile.release]` with `lto`, `codegen-units = 1`, `panic = "abort"`, `strip` matches hyalo and produces a smaller release binary at the cost of longer release builds (iter-23) |
| 2026-05-09 | Embedded syslog receiver crate named `bunny-syslog-receiver` (not `bunny-net-api-syslog`) | The `bunny-net-api-*` prefix is reserved for crates wrapping bunny.net REST surfaces. This crate is a transport library (RFC 5424 TCP listener used by `hoppy container logs`), hence the different prefix (iter-24) |
| 2026-05-09 | Expose full Optimizer surface in iter-26 in one shot | All 24 Optimizer fields live on a single `PullZoneModel`; partial coverage (e.g. only master switch + quality) would be more confusing than no coverage, forcing operators to fall back to raw `curl` for missing flags. The fields are all mechanical (same struct, same update endpoint) so doing them in one iteration costs little extra (iter-26) |
| 2026-05-10 | Storage local-file flag: `--file` everywhere | Both `storage upload` and `storage download` use `--file` for the local path (input on upload, output destination on download — stdout if omitted). Renamed `download --output` → `--file`. The earlier asymmetry (`upload --file` / `download --output`) forced operators to memorise per-direction flags. Pairs with the `--remote-path` standardisation across all storage subcommands (`ls` was `--path`) (iter-28) |

## Related
- [[development-roadmap]] — iteration history
- [[api/bunny-net-api-client-patterns]] — established API client patterns
- [[iterations/iteration-1-code-review]] — code review that drove several decisions
