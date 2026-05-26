---
title: Iter-35 — CLI discoverability (drill-down hints + lean README)
type: iteration
date: 2026-05-14
tags:
  - iteration
  - cli
  - ux
  - docs
status: in-progress
branch: iter-35/cli-discoverability
---

# Iter-35 — CLI discoverability

After the iter-27/28 dogfooding round, the remaining open backlog items are
both about helping users (and LLMs) find the next useful command. Bundling
them because they touch the same surface: post-command output and the
landing-page README.

## Scope

### 1. Drill-down hints after commands
Source: [[../backlog/drill-down-hints]]

Borrow the hyalo iter-107 pattern. After each command, print a short
`tip: <next command>` block suggesting one or two natural follow-ups.

- [x] Add a `--no-hints` global flag (default off). `--format json` implies
      `--no-hints` so machine output stays clean.
- [x] Define a small `Hint` helper in `crates/hoppy-cli/src/output/` (or
      similar) that renders to stderr after the main result.
- [x] Wire hints into the highest-value commands first:
  - `pull-zone list` → `pull-zone get --id <id>`, `pull-zone statistics --id <id>`
  - `pull-zone create` → `pull-zone hostname add`, `pull-zone edge-rule add`
  - `stream library list` → `stream video list --library <id>`
  - `container app create` → `container template add --app-id <id>`,
    `container endpoint add --app-id <id>`
  - `auth check` → `pull-zone list` (read-only smoke test)
- [x] Unit test: a command invoked with `--no-hints` must not print to stderr
      beyond its own diagnostics.
- [x] Unit test: `--format json` invocation produces no hint output.

### 2. Git-sha + date in `hoppy -V`

Reference: ff-rdp commit `4c01f0d` (build.rs at `crates/ff-rdp-cli/build.rs`,
`build_version_string()` in `crates/ff-rdp-cli/src/cli/args.rs`).

Embed the build's short git SHA and commit date into the binary so
`hoppy -V` prints e.g. `hoppy 0.3.0 (abc123def456 2026-05-26)` instead of
just `hoppy 0.3.0`. Helps dogfooding triage: "which build am I on?".

- [x] Add `crates/hoppy-cli/build.rs` that shells out to git for short SHA
      (`git rev-parse --short=12 HEAD`) and commit date
      (`git show -s --format=%cs HEAD`), emits via `cargo:rustc-env=` as
      `HOPPY_BUILD_VERSION_SHA` and `HOPPY_BUILD_DATE`. Suffix `+dirty`
      when `git status --porcelain` is non-empty.
- [x] CI/tarball escape hatches: respect `GIT_COMMIT` / `GIT_COMMIT_DATE`
      env vars if set; emit empty strings on no-git or when
      `CARGO_HOPPY_FORCE_NO_GIT=1`.
- [x] `cargo:rerun-if-changed` for `<git-dir>/HEAD` and `<git-dir>/refs/`
      (derive via `git rev-parse --git-dir` to support worktrees);
      `cargo:rerun-if-env-changed` for the three env vars above.
- [x] Add a `build_version_string()` helper in the CLI args module that
      formats `"{PKG} ({SHA} {DATE})"` when SHA is non-empty, else just
      `PKG`. Wire into `#[command(version = ...)]` on the clap parser.
- [x] Unit test: with `CARGO_HOPPY_FORCE_NO_GIT=1`, `hoppy -V` prints the
      bare `CARGO_PKG_VERSION`.
- [x] Unit test (or integration): with git available, the version output
      matches `^hoppy \d+\.\d+\.\d+ \([0-9a-f]{12}(\+dirty)? \d{4}-\d{2}-\d{2}\)$`.

### 3. Lean README
Source: [[../backlog/lean-readme]]

Restructure the top-level `README.md` as a landing page. Move exhaustive
reference into `hoppy-knowledgebase/cli/` and/or `docs/MANUAL.md`.

- [x] Hero: one paragraph + one runnable example.
- [x] Install section: `brew`, `cargo install`, deb/rpm — short blocks only.
- [x] Quick start: 3–5 commands that produce visible value
      (auth check, pull-zone list, pull-zone create, …).
- [x] Link out to `hoppy-knowledgebase/cli/command-tree.md` for the full
      surface, and to dash.bunny.net for concept docs.
- [x] Move any exhaustive sections that don't belong on a landing page
      into `docs/MANUAL.md` (new) or the knowledgebase.

## Out of scope

- New CLI commands or API surface.
- Changing `--format` semantics beyond suppressing hints on json.

## Acceptance

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] Dogfooding pass: run the five hinted commands and confirm the tips
      are accurate and actionable.
- [x] First-time reader can find install + quick start on the README in
      under 30 seconds.

## Related

- [[../backlog/drill-down-hints]]
- [[../backlog/lean-readme]]
- hyalo iter-107 commit `d28325d` (drill-down hints reference)
- hyalo commit `4b6df49` (lean README reference)
