---
title: Iter-60 — dns zone export format + empty-zone output
type: iteration
date: 2026-06-01
tags:
  - iteration
  - dns
  - format
  - polish
status: completed
branch: iter-60/dns-export-format
---

# Iter-60 — DNS zone export format and empty-zone output

## Why

Two related polish items on the same surface:

1. `dns zone export --format json` ignores `--format` entirely and
   emits raw BIND text regardless.
2. `dns zone export` on an empty zone produces literally nothing
   (no header, no blank line) — silent success looks identical to
   a hang.

See [[../backlog/dns-zone-export-ignores-format]] and
[[../backlog/dns-zone-export-empty-zone-silent]].

## Scope

### 1. Decide the format mapping [1/1]

- [x] Pick the JSON shape:
      (a) **Envelope**: `{"Bind": "<BIND text>"}` — minimal, ships
          the same payload but JSON-wrapped.
      (b) **Structured**: `{"Records": [{...}, ...]}` — pre-parsed.
      Recommended: **(a)** for parity with [[iteration-56-db-format-cleanup]]
      and minimal cost. Document the decision in the PR.

### 2. Implement [3/3]

- [x] Route `dns zone export` through the standard `--format`
      pipeline.
- [x] `--format json` emits the chosen envelope. `--format text`
      keeps the existing raw BIND output (current default
      behaviour). `--format table` either renders a per-record
      table or aliases to text — pick and document.
- [x] For empty zones, emit at minimum a `;; zone <domain> — 0 records`
      comment so stdout is never literally empty.

### 3. Tests [3/3]

- [x] E2E mock test for `--format json` on a zone with records.
- [x] E2E mock test for empty-zone behaviour across all three
      formats.
- [x] Snapshot test for `--format text` keeping current output.

## Out of scope

- `dns zone import` — separate surface.
- Per-record structured JSON — explicitly chosen against in step 1
  unless the user asks otherwise.

## Acceptance Criteria

- [x] `dns zone export --format json` emits valid JSON for both
      populated and empty zones.
- [x] `dns zone export --format text` keeps the current raw BIND
      output for populated zones.
- [x] Empty-zone exports produce non-empty stdout across all
      formats.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/dns-zone-export-ignores-format]]
- [[../backlog/dns-zone-export-empty-zone-silent]]
- [[../iterations/iteration-56-db-format-cleanup]]
- [[../dogfooding/session-2026-06-01-round2]]
