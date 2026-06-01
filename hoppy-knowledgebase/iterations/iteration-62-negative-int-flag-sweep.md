---
title: Iter-62 — Negative-int flag parsing sweep
type: iteration
date: 2026-06-01
tags:
  - iteration
  - cli
  - clap
  - dx
status: planned
branch: iter-62/negative-int-flag-sweep
---

# Iter-62 — Negative-int flag parsing sweep

## Why

```sh
$ hoppy container app create --min -1 --max 1 ...
error: unexpected argument '-1' found
```

Clap treats `-1` as a short flag, not a value for `--min`. The user
sees a confusing parse error instead of a domain validation. This
trip-wire exists on every numeric flag that could plausibly be
negative.

See [[../backlog/container-app-create-negative-int-rejection]].

## Scope

### 1. Audit numeric flags [0/2]

- [ ] Grep the workspace for `#[arg(...)] ... <i32|i64|isize|...>`
      and similar numeric flag declarations. Build a list per
      subcommand.
- [ ] Mark each as "domain allows negative" (e.g. priorities,
      offsets, deltas) or "domain forbids negative" (counts,
      ports, IDs).

### 2. Implement [0/2]

- [ ] For flags whose domain allows negative values: add
      `allow_hyphen_values = true` so `--min -1` parses cleanly,
      then validate the range in the handler with a clear error
      ("min must be in [0, N]").
- [ ] For flags whose domain forbids negative values: keep the
      current clap behaviour but extend the error rendering so
      "unexpected argument '-1'" becomes "negative values are not
      accepted for --<flag>; use --<flag>=<n> or a non-negative
      value".

### 3. Tests [0/2]

- [ ] E2E test: `container app create --min -1` produces a domain
      validation error.
- [ ] E2E test: a flag that legitimately accepts negative values
      (if any survive step 1) parses `--<flag> -<n>` cleanly.

## Out of scope

- Rewriting CLI parsing wholesale.
- The unrelated `--min` / `--max` cross-field validation already
  surfaced upstream by the API.

## Acceptance Criteria

- [ ] All numeric flags across the workspace either accept negative
      values cleanly or surface a clap-aware error explaining the
      issue.
- [ ] No subcommand surfaces "unexpected argument '-1'" or similar
      for a numeric flag that the user obviously meant as a value.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/container-app-create-negative-int-rejection]]
- [[../dogfooding/session-2026-06-01-round2]]
