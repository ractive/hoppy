---
title: Iter-52 — expose Stream library API keys via CLI
type: iteration
date: 2026-06-01
tags:
  - iteration
  - stream
  - secrets
  - dx
status: in-progress
branch: iter-52/stream-library-api-keys
---

# Iter-52 — expose Stream library API keys

## Why

The bunny.net API returns `ApiKey` and `ReadOnlyApiKey` on the Stream
library object, but hoppy has no path to view them — not even with
`--reveal`. Users have to go to the web UI to copy keys they already
have programmatic access to.

See [[../backlog/stream-library-api-key-unrecoverable]].

## Scope

### 1. Confirm the data path

- [x] Verify the upstream `stream library list`/`get` responses
      contain `ApiKey` and `ReadOnlyApiKey` (capture a fixture).
- [x] Confirm whether they require a separate keys endpoint or come
      back inline on `get`.

### 2. Add fields to the Rust type

- [x] `api_key: Option<String>` and `read_only_api_key: Option<String>`
      on the `StreamLibrary` (read) type with explicit `#[serde(rename)]`.
- [x] Mark them as sensitive (see step 3).

### 3. Reveal flag wiring

- [x] `hoppy stream library get` and `... list` redact these by
      default (print `***` or omit).
- [x] `--reveal` flag (or reuse the existing one if there is one)
      prints them in full across all formats.
- [x] `--format json` with `--reveal` includes them; without
      `--reveal`, redact.

### 4. Tests

- [x] Snapshot test with `--reveal` shows real key.
- [x] Snapshot test without `--reveal` shows redacted value.
- [x] E2E live test (feature `live-api`) confirms the field is
      populated.

### 5. Docs

- [x] Note the `--reveal` requirement in command help text and
      the dogfooding playbook.

## Out of scope

- Rotating / generating new API keys via the CLI (separate iteration).
- Storage zone access keys (different shape).

## Acceptance Criteria

- [x] `hoppy stream library get <id> --reveal` prints `ApiKey` and
      `ReadOnlyApiKey` in text/json output.
- [x] Default output (no `--reveal`) hides them.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/stream-library-api-key-unrecoverable]]
