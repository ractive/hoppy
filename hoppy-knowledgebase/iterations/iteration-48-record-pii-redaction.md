---
title: Iter-48 — record-mode PII redaction
type: iteration
date: 2026-06-01
tags: [iteration, security, recording, fixtures, dx]
status: planned
branch: iter-48/record-pii-redaction
---

# Iter-48 — record-mode PII redaction

## Why

`hoppy --record <dir>` writes raw API responses straight to disk. The
dogfooding playbook tells users to run the live test suite with
`HOPPY_RECORD_DIR=…` and **commit the drift**. The very first command
in the playbook (`auth check`) calls `GET /billing` and persists:

- Real balance + monthly charges
- Payer email
- Payment IDs
- Signed invoice download URLs with embedded tokens

This is a security issue: the playbook tells users to commit the
output, and signed URLs would remain hot until they expire. See
[[../backlog/record-flag-leaks-billing-pii]].

## Scope

### 1. Build a redaction layer

- [ ] New module `crates/bunny-net-api/src/recording/redact.rs` (or in
      whichever crate owns `--record`).
- [ ] Operates on `serde_json::Value` before write — field-name + value
      pattern based.
- [ ] Centralised redaction config: one list of field-name patterns,
      one list of value-shape patterns (signed URLs, JWTs).

### 2. Redaction rules

- [ ] Field-name patterns to mask (case-insensitive substring):
      `email`, `payer`, `paymentid`, `balance`, `charges`, `recharge`,
      `invoice`, `downloadurl`, `apikey`, `accesskey`, `token`,
      `password`.
- [ ] Value patterns to mask: URLs containing `?token=` / `&token=`
      / `signature=` / `expires=`; JWT-shape strings.
- [ ] Replacement: `"<redacted>"` for strings, `0` for numbers,
      preserve array/object shape so fixture diffs remain meaningful.

### 3. Plumbing

- [ ] Apply redaction in the recording write path (where bytes hit
      disk), not at the API-response decode site.
- [ ] Add `--no-redact` escape hatch for the rare case where a
      developer needs the raw payload (off by default, documented as
      "do not commit").

### 4. Tests

- [ ] Unit tests for `redact()` covering each rule.
- [ ] E2E test: record a synthetic billing-shaped response, assert all
      sensitive fields are masked and structure is preserved.
- [ ] Snapshot test: feed a known-good billing JSON in, assert
      redacted output matches a checked-in fixture.

### 5. Docs

- [ ] Update [[../dogfooding/dogfooding-playbook]] to note that
      redaction is on by default and what fields it covers.
- [ ] Add a short note to the recording docs that fixture diffs
      should still be spot-checked before commit.

## Out of scope

- Redacting *existing* recorded fixtures on disk (separate sweep if
  desired).
- Per-domain redaction overrides — global ruleset is enough for now.

## Acceptance Criteria

- [ ] `hoppy --record /tmp/x auth check` produces a
      `core/GET_billing.json` with no real balance, email, payment ID,
      or signed URL.
- [ ] Existing live-API tests still pass with redaction on.
- [ ] `--no-redact` flag works and is documented.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/record-flag-leaks-billing-pii]]
- [[../backlog/fixture-recording-name-mismatch]]
- [[../dogfooding/dogfooding-playbook]]
