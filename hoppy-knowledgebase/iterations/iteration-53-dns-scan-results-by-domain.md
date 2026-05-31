---
title: Iter-53 — DNS scan results by domain / job-id
type: iteration
date: 2026-06-01
tags: [iteration, dns, scan, onboarding, dx]
status: planned
branch: iter-53/dns-scan-results-by-domain
---

# Iter-53 — DNS scan results by domain

## Why

`hoppy dns zone scan start --domain <d>` returns a `JobId` for a
domain not yet onboarded, but `hoppy dns zone scan results` only
accepts `--id <zone-id>` — there is no zone yet, and `--job-id` is
not a flag. The canonical "scan before onboarding" workflow has no
follow-up.

See [[../backlog/dns-scan-results-rejects-domain]].

## Scope

### 1. Pick the resolution path

- [ ] Investigate the API: does `scan results` take a domain, a
      job id, or both? Document in the PR.
- [ ] If the API needs a job id we must persist, decide whether to
      add a local cache or require the caller to keep the id.

### 2. Implement

- [ ] Add `--domain <d>` and/or `--job-id <id>` to
      `hoppy dns zone scan results`, mirroring what `start` accepts.
- [ ] Keep `--id` for the existing-zone case.
- [ ] Clap arg group ensures exactly one of `--id` / `--domain` /
      `--job-id` is supplied.

### 3. Improve `scan start` output

- [ ] Print a hint after `start` showing the exact next command,
      e.g. `Run: hoppy dns zone scan results --job-id <id>`.
- [ ] Hint suppressed for `--format json`.

### 4. Tests

- [ ] E2E snapshot for `scan start --domain` then `scan results
      --job-id` flow.
- [ ] Error-case snapshot when no arg supplied.
- [ ] Live test (feature `live-api`) end-to-end.

## Out of scope

- Onboarding (creating the zone) from the scan result — separate
  follow-up.
- Polling/retry helpers — caller scripts can loop.

## Acceptance Criteria

- [ ] `scan start --domain <d>` followed by `scan results --domain <d>`
      (or `--job-id`) returns the scan output without manual zone
      creation.
- [ ] Existing `--id` path still works.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/dns-scan-results-rejects-domain]]
