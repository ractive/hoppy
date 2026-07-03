---
title: Iter-57 — Define and enforce the --quiet flag contract
type: iteration
date: 2026-06-01
tags:
  - iteration
  - cli
  - global-flags
  - dx
  - consistency
status: completed
branch: iter-57/quiet-flag-contract
---

# Iter-57 — Define and enforce the `--quiet` flag contract

## Why

`--quiet` is exposed as a global flag on every subcommand and
documented as "Suppress non-essential output", but on `auth check`
(and likely many other read commands) it's a no-op — the full
table is still printed.

See [[backlog/quiet-flag-no-op-on-many-commands]].

## Scope

### 1. Decide the contract [1/1]

- [x] Pick one of:
      (a) **Strict**: `--quiet` only suppresses ancillary lines
          (drill-down hints, "Saved to …" confirmation prints,
          progress bars). Primary payload always prints.
      (b) **Liberal**: on read commands with a non-zero exit on
          failure (`auth check`, etc.), `--quiet` suppresses the
          entire stdout payload on success and only prints on
          error. Useful in shell scripts.
      Recommended: **(b)** for the predicate-style commands
      (`auth check`, `db ping`), **(a)** everywhere else.

### 2. Audit the surface [2/2]

- [x] Walk every subcommand and classify it as "predicate"
      (success/failure is the entire signal) vs "data" (the
      payload is the point). Record the classification in
      `cli/quiet-flag-classification.md`.
- [x] Identify which commands currently print ancillary lines
      (hints, "Deleted …" confirmations, etc.) that `--quiet`
      should suppress under the strict reading.

### 3. Implement [3/3]

- [x] Plumb the `quiet` flag through the print/hint helpers so
      "non-essential" lines are gated.
- [x] For predicate commands, suppress the entire stdout payload
      under `--quiet` and rely on exit code.
- [x] Hide `--quiet` from `--help` on any command where it is a
      genuine no-op after the audit (or remove the unused arg).

### 4. Tests [2/2]

- [x] E2E test: `auth check --quiet` on success prints nothing,
      exits 0; on failure prints the error, exits non-zero.
- [x] E2E test: a sample "data" command (`pull-zone list --quiet`)
      still prints its primary table but skips the drill-down
      hint.

## Out of scope

- `--no-hints` is already a separate flag and stays as-is.
- Restructuring stderr/stdout split beyond what's needed for
  `--quiet`.

## Acceptance Criteria

- [x] Every subcommand's `--quiet` behaviour is documented in
      `cli/quiet-flag-classification.md` (or equivalent).
- [x] No subcommand exposes `--quiet` in `--help` without
      implementing it.
- [x] `hoppy auth check --quiet && echo OK || echo FAIL` prints
      `OK` on a valid key and `FAIL` on an invalid one, with no
      other output on success.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[backlog/quiet-flag-no-op-on-many-commands]]
- [[dogfooding/session-2026-06-01]]
- [[cli/quiet-flag-classification]] — per-subcommand classification recorded during this iteration
