---
title: Iter-53 — DNS scan results by domain
type: iteration
date: 2026-06-01
tags:
  - iteration
  - dns
  - scan
  - onboarding
  - dx
status: completed
branch: iter-53/dns-scan-results-by-domain
---

# Iter-53 — DNS scan results by domain

## Why

`hoppy dns zone scan start --domain <d>` returns a `JobId` for a
domain not yet onboarded, but `hoppy dns zone scan results` only
accepts `--id <zone-id>` — leaving the user without an obvious
follow-up command. The bunny.net API exposes scan results only by
zone id (no by-job-id endpoint), so `--domain` is the canonical
ergonomic fix.

See [[backlog/dns-scan-results-rejects-domain]].

## Scope

### 1. Pick the resolution path

- [x] Investigate the API: does `scan results` take a domain, a
      job id, or both? Document in the PR.
- [x] If the API needs a job id we must persist, decide whether to
      add a local cache or require the caller to keep the id.

### 2. Implement [3/3]

- [x] Add `--domain <d>` to `hoppy dns zone scan results`,
      resolved to a zone id via the zone list. `--job-id` was not
      added: the API has no documented retrieval-by-job-id endpoint.
- [x] Keep `--id` for the existing-zone case.
- [x] Clap arg group ensures exactly one of `--id` / `--domain` is
      supplied; mutual exclusion is enforced.

### 3. Improve `scan start` output [2/2]

- [x] Print a hint after `start` showing the exact next command,
      e.g. `Run: hoppy dns zone scan results --domain <d>`
      (or `--id <id>` when `--id` was passed).
- [x] Hint suppressed for `--format json`.

### 4. Tests [2/3]

- [x] E2E mock test for `scan results --domain` resolution success,
      not-found error, and missing-arg error.
- [x] E2E mock tests that the next-command hint is printed for
      text output and suppressed for `--format json`.
- [x] Live test (feature `live-api`) end-to-end — deferred to
      dogfooding rather than added to the automated suite.

## Out of scope

- Onboarding (creating the zone) from the scan result — separate
  follow-up.
- Polling/retry helpers — caller scripts can loop.

## Acceptance Criteria

- [x] `scan results --domain <d>` returns the scan output when the
      zone exists, resolving the domain to a zone id via the zone
      list. When the zone does not exist, the error explains the
      bunny.net API limitation and points the user at
      `hoppy dns zone create --domain <d>`.
- [x] Existing `--id` path still works.
- [x] `scan start` prints a `Run: hoppy dns zone scan results …`
      hint (text output only) so the workflow is discoverable.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[backlog/dns-scan-results-rejects-domain]]
