---
title: Iter-54 — Shield metrics flag parity (waf-rule --id)
type: iteration
date: 2026-06-01
tags:
  - iteration
  - shield
  - metrics
  - dx
  - consistency
status: planned
branch: iter-54/shield-metrics-flag-parity
---

# Iter-54 — Shield metrics flag parity

## Why

Every `shield metrics <sub>` subcommand uses `--id` for the Shield Zone
ID, except `shield metrics waf-rule`, which uses `--shield-zone-id`.
The divergence trips users up the first time they reach for the
sibling pattern.

See [[../backlog/shield-metrics-waf-rule-flag-divergence]].

## Scope

### 1. Pick the convention [1/1]

- [ ] Decide: rename `--shield-zone-id` to `--id` on `waf-rule`
      (matches all siblings) and keep `--shield-zone-id` as a
      hidden alias for back-compat.

### 2. Implement [0/2]

- [ ] Update `shield metrics waf-rule` clap arg: `long = "id"` with
      `alias = "shield-zone-id"`.
- [ ] Verify no other `shield metrics *` subcommand carries the old
      name; if it does, apply the same alias.

### 3. Tests [0/2]

- [ ] E2E mock test calling `shield metrics waf-rule --id <z>
      --rule-id <r>` succeeds.
- [ ] E2E mock test that `--shield-zone-id` still works (hidden
      alias).

## Out of scope

- Renaming `--rule-id` (already unambiguous).
- Touching other `shield <noun>` subcommand flag names.

## Acceptance Criteria

- [ ] `hoppy shield metrics waf-rule --id <z> --rule-id <r>` works.
- [ ] `hoppy shield metrics waf-rule --shield-zone-id <z> --rule-id <r>`
      still works (hidden alias).
- [ ] `--help` shows `--id` as the documented flag.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/shield-metrics-waf-rule-flag-divergence]]
- [[../dogfooding/session-2026-06-01]]
