---
title: Iter-64 — Empty --help string sweep
type: iteration
date: 2026-06-01
tags:
  - iteration
  - cli
  - help
  - polish
status: completed
branch: iter-64/empty-help-strings-sweep
---

# Iter-64 — Empty `--help` string sweep

## Why

`pull-zone create --name` exposes no help description — every other
flag on the subcommand does. The empty body reads like a TODO and
is the kind of thing CI should catch.

See [[backlog/pull-zone-create-name-help-empty]].

## Scope

### 1. Audit [1/1]

- [x] Grep the workspace for `#[arg(...)]` declarations without a
      `help = "..."` (or a doc comment that clap uses). Build the
      list — at minimum `pull-zone create --name`; probably more.

### 2. Implement [2/2]

- [x] Write a one-liner help description for each flag in the
      audit list. For `pull-zone create --name`, use something
      close to:
      > Pull Zone name. Becomes the hostname `<name>.b-cdn.net`
      > and must be globally unique across bunny.net. Lowercase
      > letters, digits, and hyphens only.
- [x] Match tone to surrounding flags on the same subcommand.

### 3. Tests [2/2]

- [x] Add a workspace-level unit test that walks every clap arg
      and asserts `help.is_some() && !help.unwrap().is_empty()`.
      Future regressions get caught in CI.
- [x] Snapshot check (or grep) on representative `--help` outputs
      to confirm the new descriptions render.

## Out of scope

- Rewording existing help strings that aren't empty (separate
  polish pass).
- Internationalisation.

## Acceptance Criteria

- [x] `hoppy pull-zone create --help` shows a non-empty description
      for `--name`.
- [x] The workspace unit test passes (no flag has empty help).
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[backlog/pull-zone-create-name-help-empty]]
- [[dogfooding/session-2026-06-01-round2]]
