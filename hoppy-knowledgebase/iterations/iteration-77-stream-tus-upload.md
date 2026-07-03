---
title: Iter-77 — stream TUS resumable upload
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - stream
  - upload
status: planned
branch: iter-77/stream-tus-upload
priority: 4
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/stream
---

# Iter-77 — stream TUS resumable upload

## Why

Per [[research/api-coverage-2026-07/stream]] §4, bunny Stream supports
TUS resumable uploads (`video.bunnycdn.com/tusupload`,
signature-based auth) but it is docs-only — no spec, no client, no CLI.
For large files over flaky links the single-shot PUT is the only option
today. Biggest single feature of the plan; scheduled last for that
reason.

## Scope

### 1. TUS protocol client

- [ ] Implement a TUS 1.0 client in `crates/bunny-net-api/src/stream/`
  (or a `tus` submodule): creation request, `HEAD` offset probe,
  `PATCH` chunk upload with `Upload-Offset` handling, per
  `docs.bunny.net/stream/tus-resumable-uploads` (docs-only, no
  OpenAPI spec)
- [ ] Signature-based auth (library ID + API key + expiry hash) as the
  docs describe; pre-signed metadata headers for library/video IDs
- [ ] Chunked streaming reads — never buffer the whole file (project
  performance rule)

### 2. CLI surface

- [ ] `stream video upload --resumable` (or a dedicated subcommand if
  flag semantics get muddy) — reuse the existing create-then-upload
  composite shape
- [ ] Retry with backoff on transient failures; resume from the server
  offset after interruption
- [ ] Offset/session persistence on disk so a re-run resumes an
  interrupted upload; state file location must be Windows/Linux/macOS
  safe (`std::path::PathBuf`, no Unix-only assumptions)
- [ ] Progress bar consistent with the existing PUT upload path

### 3. Per-upload params integration

- [ ] The per-upload params from
  [[iteration-69-filters-pagination-sweep]] (`jitEnabled`,
  `enabledResolutions`, `enabledOutputCodecs`, `transcribe*`,
  `generate*`, `sourceLanguage`) must also apply on the TUS path
  (metadata headers) — same flags, both transports

### 4. Tests

- [ ] Unit tests against a wiremock/minimal TUS server: offset resume,
  mid-upload interruption, checksum of assembled payload
- [ ] e2e test for the new flag surface (`tests/e2e/` pattern)
- [ ] Live dogfood with a large file + forced interruption; note
  friction in the KB

## Out of scope

- The 13 stale live-streaming endpoints in the Mintlify stream mirror —
  explicitly do-not-implement (gap analysis §1 caveat)
- Parallel/multi-connection chunk upload — resume correctness first

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover the new/changed upload commands (`tests/e2e/` pattern)
- [ ] Interrupted-then-resumed upload verified live (dogfooding playbook)
- [ ] Help text updated for the resumable upload surface
- [ ] `hyalo lint` clean on touched knowledgebase files
