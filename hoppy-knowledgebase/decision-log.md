---
title: "Decision Log"
date: 2026-03-20
tags:
  - decisions
  - architecture
status: active
---

# Decision Log

Significant architectural and design decisions made during development. API-specific quirks are documented separately in [[api/bunny-api-quirks]].

## Architecture

| Decision | Rationale |
|----------|-----------|
| Hand-written API clients (not codegen) | Progenitor codegen produced 51K lines vs 4K hand-written. PascalCase API made codegen awkward. See [[research/hand-written-experiment-results]] |
| No `--api-key` flag | clig.dev: don't pass secrets via flags (visible in ps/history). `BUNNY_API_KEY` env var only |
| Table output as default for TTY | Follows az/gcloud/clig.dev pattern. JSON auto-default when piped |
| Branch per iteration | Keeps main stable. Branch naming: `iter-N/description`. Merge via PR |
| PaginatedList/ApiError kept separate per crate | Intentionally duplicated across bunny-api-core and bunny-api-compute — independent workspace members, shared extraction would add coupling without benefit |
| Magic Containers API hand-written from docs | No OpenAPI spec available. 47 endpoints. Uses camelCase serde (like Shield), cursor-based pagination, ProblemDetails+ErrorDetails error handling |

## Security

| Decision | Rationale |
|----------|-----------|
| `zone_security_key` excluded from JSON output | `#[serde(skip_serializing)]` prevents leaking security keys in CLI output |
| `VideoLibrary.ApiKey` excluded from JSON output | Same pattern — `api_key` and `read_only_api_key` skipped |
| `deployment_key` excluded from JSON output | Same pattern for EdgeScript credentials |

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
| Homebrew tap: `ractive/homebrew-hoppy` | Single-formula tap keeps things isolated |
| Shell completions: stdout-only | Industry standard (starship, rustup, gh). Package managers handle installation |
| cross-rs for Linux aarch64 only | Native runners for everything else. cross-rs more reliable than cargo-zigbuild |
| Windows aarch64 included | Builds natively on `windows-latest` runner — zero extra effort |
