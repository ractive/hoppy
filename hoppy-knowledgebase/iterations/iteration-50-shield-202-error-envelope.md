---
title: Iter-50 — surface Shield 202 error envelope
type: iteration
date: 2026-06-01
tags:
  - iteration
  - shield
  - error-handling
  - dx
status: in-progress
branch: iter-50/shield-202-error-envelope
---

# Iter-50 — surface Shield 202 error envelope

## Why

Bunny's Shield API returns HTTP 202 with an error body when the
account isn't on a high-enough tier. Hoppy parses 202 as success
with no results, so the user sees "No results" instead of the real
"upgrade required" message that the API gave back.

See [[../backlog/shield-202-error-swallowed]].

## Scope

### 1. Diagnose the response shape

- [x] Capture the real 202 response body (record-mode) from a
      free-tier account against `shield event-logs`. Document the
      envelope in the PR.
- [x] Confirm whether it's a single shape or multiple (`Message`
      field, `ErrorKey`, etc.).

### 2. Fix the Shield client

- [x] In the shield API call site, treat 202-with-error-envelope as
      an error path, not an empty-result path.
- [x] Map the envelope to an `anyhow::Error` (or domain error type
      if one exists) that propagates the message text.
- [x] CLI surface: print the API message verbatim, exit non-zero.

### 3. Audit sibling endpoints

- [x] Check every other Shield subcommand for the same 202 path
      (logs, stats, anything async-ish).
- [x] Apply the same fix where the envelope appears.

### 4. Tests

- [x] Unit test: feed a captured 202+error JSON body, assert the
      client returns an error with the API message in it.
- [x] E2E snapshot test for the CLI: stub a 202 response, assert
      stderr contains the upgrade message and exit code is non-zero.

## Out of scope

- Broader 202-handling sweep across other domains (storage, stream,
  edge-rules). File follow-ups if discovered.

## Acceptance Criteria

- [x] On a free-tier account, `hoppy shield event-logs` prints the
      API's upgrade-required message and exits non-zero.
- [x] No regression for the happy-path 200 response.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/shield-202-error-swallowed]]
- [[../backlog/shield-api-error-mapping]]
- [[../backlog/debug-flag-omits-request-body]]
