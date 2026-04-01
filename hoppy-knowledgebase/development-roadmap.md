---
date: 2026-03-20
status: active
tags:
- roadmap
- planning
- iteration
title: Hoppy Development Roadmap
type: roadmap
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
| [[iterations/iteration-0-skeleton\|0]] | `iter-0/skeleton` | Project skeleton: clap CLI, output formatting, auth, error handling, CI |
| [[iterations/iteration-0.5-codegen-experiment\|0.5]] | `iter-0.5/codegen-experiment` | Evaluated Progenitor codegen → abandoned in favor of hand-written clients |
| [[iterations/iteration-1-pull-zones\|1]] | `iter-1/pull-zones` | Full Pull Zone CRUD — first vertical slice proving the stack |
| [[iterations/iteration-2-storage\|2]] | `iter-2/storage` | Storage Zones + file operations (upload/download/ls/rm), per-zone auth |
| [[iterations/iteration-3-dns\|3]] | `iter-3/dns` | DNS zones + records (all record types), 15 wiremock tests |
| [[iterations/iteration-4-stream\|4]] | `iter-4/stream` | Stream libraries + videos + collections, Stream API key resolution |
| [[iterations/iteration-5-shield\|5]] | `iter-5/shield` | Shield zones, WAF, rate limiting, access lists, bot detection, 27 wiremock tests |
| [[iterations/iteration-6-scripting-containers\|6]] | `iter-6/scripting-containers` | Edge scripting (scripts, variables, secrets, releases) + Magic Containers (47 endpoints, 57 wiremock tests) |
| [[iterations/iteration-7-cleanup\|7]] | `iter-7/cleanup` | Auth check, FromStr impls, progress bars, WAF profiles, deferred items |
| [[iterations/iteration-8-release\|8]] | `iter-8/release` | Release workflow (6 targets), Homebrew, deb/rpm, man pages, shell completions, README |
| [[iterations/iteration-9-gap-analysis\|9]] | `iter-9/gap-analysis` | Wired missing CLI commands (stream video update/fetch, collections, script rotate-key), query param redaction |
| [[iterations/iteration-10-e2e-test-harness\|10]] | `iter-10/e2e-test-harness` | 103 mock-based CLI E2E tests with assert_cmd + wiremock. **Superseded by iter 11.** |
| [[iterations/iteration-11-e2e-lifecycle\|11]] | `iter-11/e2e-lifecycle-testbooks` | Replaced iter 10 harness with wiremock + insta snapshot tests and `--features live-api` lifecycle tests |
| [[iterations/iteration-12-api-coverage\|12]] | `iter-12/api-coverage-gaps` | URL purge, pull zone hostnames/SSL, DNS export/import, stream captions, shield metrics overview (12 new API methods, 10 new CLI commands) |

---

## Current State (post iter 12)

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
- [ ] Containers live E2E tests — deferred (cost/complexity)

---

## Planned Iterations (API Coverage)

| Iter | Branch | Summary | New API Methods | Complexity |
|------|--------|---------|----------------|------------|
| [[iterations/iteration-13-statistics\|13]] | `iter-13/statistics` | Statistics & analytics across all services (shield detailed metrics, storage/DNS/pull zone/stream stats) | 11 | Medium |
| [[iterations/iteration-14-edge-rules\|14]] | `iter-14/edge-rules` | Pull zone edge rules — custom request/response handling at CDN edge (add/update/delete/enable rules with triggers and actions) | 3 | Medium |
| [[iterations/iteration-15-pullzone-access-control\|15]] | `iter-15/pullzone-access-control` | Pull zone referrer allow/block lists and IP blocking (6 endpoints, 8 CLI commands) | 6 | Small |
| [[iterations/iteration-16-stream-video-processing\|16]] | `iter-16/stream-video-processing` | Stream video processing — transcription, heatmaps, re-encoding, thumbnails, resolution management, storage size | 10 | Medium |
| [[iterations/iteration-17-dns-security\|17]] | `iter-17/dns-security` | DNS security — DNSSEC management, wildcard certificate issuance, DNS record scanning | 5 | Medium |
| [[iterations/iteration-18-shield-advanced\|18]] | `iter-18/shield-advanced` | Shield advanced — API Guardian, upload scanning, event logs, WAF triggered rule review | 16 | Medium-Large |

**Total across iters 13–18:** ~51 new API methods, ~47 new CLI commands

### Excluded from planned iterations

- **Security credential rotation** (resetSecurityKey, resetPassword, resetApiKey) — too destructive for CLI automation
- **Billing summary/PDF downloads** — low CLI value, better in dashboard
- **Video library watermark/referrer management** — niche, can add later
- **Account management** (close account, audit logs) — dangerous/niche
- **OEmbed** — embed HTML generation, not a CLI operation

---

## Possible Future Iterations (Non-API-Coverage)

- **Config file support** — `~/.config/hoppy/config.toml` for defaults (API key, default format, etc.)
- **`--dry-run` for mutating operations** — show what would happen without executing
- **`--wait` for async operations** — poll until operation completes
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
| 6 — Scripting + Containers | Two services, API client + CLI for both | Medium-Large |
| 7 — Code Cleanup | Tech debt, small features, deferred items | Small |
| 8 — Release Readiness | CI/CD, packaging, docs, Homebrew | Medium |
| 9 — Gap Analysis | Wire missing CLI commands from audit | Small |
| 10 — E2E Test Harness | Mock-based CLI tests with wiremock | Medium |
| 11 — Lifecycle Tests | Live API lifecycle tests with snapshots | Medium |
| 12 — API Coverage Gaps | URL purge, hostnames/SSL, DNS export/import, captions, shield metrics | Medium |
| 13 — Statistics | Stats/analytics across all services | Medium |
| 14 — Edge Rules | Pull zone edge rules (triggers + actions) | Medium |
| 15 — Access Control | Pull zone referrer/IP blocking | Small |
| 16 — Video Processing | Stream transcription, heatmaps, re-encoding, thumbnails | Medium |
| 17 — DNS Security | DNSSEC, certificates, record scanning | Medium |
| 18 — Shield Advanced | API Guardian, upload scanning, event logs, WAF review | Medium-Large |

## Related
- [[Seed]] — original project brief
- [[testing/test-plan-v0.1.0]] — comprehensive pre-release test plan
- [[iterations/iteration-1-code-review]] — code review from iter 1
- [[release/release-setup-checklist]] — release setup steps
