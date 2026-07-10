---
title: Iter-70 — log retrieval services (cdn-logging + origin-errors)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - logging
status: in-progress
branch: iter-70/log-retrieval-services
priority: 2
related:
  - research/api-coverage-gap-analysis-2026-07
---

# Iter-70 — log retrieval services

## Why

Per [[research/api-coverage-gap-analysis-2026-07]] §1–2, the cdn-logging
(2 ops, `logging.bunnycdn.com`) and origin-errors (1 op) services are
100% missing — two entire services with 0% coverage, but trivially small
modules. Tiny effort, whole-service coverage win.

## Scope

### 1. `logging` API module

- [x] New feature-gated module `crates/bunny-net-api/src/logging/`
  covering both ops in `specs/logging.json` (pull-zone access log
  retrieval); declare the `logging` feature in
  `crates/bunny-net-api/Cargo.toml`, add `pub mod logging;` to `lib.rs`

### 2. `origin_errors` API module

- [x] New feature-gated module `crates/bunny-net-api/src/origin_errors/`
  covering the single op in `specs/origin-errors.json`; same feature +
  `lib.rs` wiring pattern

### 3. CLI command group

- [x] New `hoppy logs` group: `hoppy logs pull-zone` for access logs and
  `hoppy logs origin-errors` for origin error logs (naming per the gap
  analysis suggestion; adjust during implementation if the specs
  suggest better verbs)
- [x] Sensible date/zone selectors mirroring the spec params. If either
  spec exposes pagination or filter query params, wire them through in
  full on day one rather than partially — [[iteration-69-filters-pagination-sweep]]
  found several client methods that had silently dropped params since
  their original implementation, which then needed a dedicated sweep
  iteration to fix; cheaper to get complete coverage the first time

### 4. Streaming download path

- [x] Log bodies can be arbitrarily large — stream via `bytes_stream()`
  to stdout or `--output <file>`, never buffer whole payloads
  (project performance rule)

### 5. Tests & fixtures

- [x] Wiremock unit tests for both clients
- [x] e2e tests in `tests/e2e/` (new `mod` in `tests/e2e/mod.rs`,
  not top-level files)
- [x] Record fixtures under `fixtures/logging/` /
  `fixtures/origin-errors/` if live access is available

## Out of scope

- Log-forwarding configuration (already covered under
  `pull-zone update --log-forwarding-*` and `container log-forwarding`)
- Log parsing/analytics — retrieval only

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new command (`tests/e2e/` pattern)
- [x] Help text present for the new `logs` group and subcommands
- [x] `hyalo lint` clean on touched knowledgebase files
