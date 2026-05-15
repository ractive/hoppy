---
title: Iter-35 — CLI discoverability (drill-down hints + lean README)
type: iteration
date: 2026-05-14
tags:
  - iteration
  - cli
  - ux
  - docs
status: planned
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

- [ ] Add a `--no-hints` global flag (default off). `--format json` implies
      `--no-hints` so machine output stays clean.
- [ ] Define a small `Hint` helper in `crates/hoppy-cli/src/output/` (or
      similar) that renders to stderr after the main result.
- [ ] Wire hints into the highest-value commands first:
  - `pull-zone list` → `pull-zone get --id <id>`, `pull-zone statistics --id <id>`
  - `pull-zone create` → `pull-zone hostname add`, `pull-zone edge-rule add`
  - `stream library list` → `stream video list --library <id>`
  - `container app create` → `container template add --app-id <id>`,
    `container endpoint add --app-id <id>`
  - `auth check` → `pull-zone list` (read-only smoke test)
- [ ] Unit test: a command invoked with `--no-hints` must not print to stderr
      beyond its own diagnostics.
- [ ] Unit test: `--format json` invocation produces no hint output.

### 2. Lean README
Source: [[../backlog/lean-readme]]

Restructure the top-level `README.md` as a landing page. Move exhaustive
reference into `hoppy-knowledgebase/cli/` and/or `docs/MANUAL.md`.

- [ ] Hero: one paragraph + one runnable example.
- [ ] Install section: `brew`, `cargo install`, deb/rpm — short blocks only.
- [ ] Quick start: 3–5 commands that produce visible value
      (auth check, pull-zone list, pull-zone create, …).
- [ ] Link out to `hoppy-knowledgebase/cli/command-tree.md` for the full
      surface, and to dash.bunny.net for concept docs.
- [ ] Move any exhaustive sections that don't belong on a landing page
      into `docs/MANUAL.md` (new) or the knowledgebase.

## Out of scope

- New CLI commands or API surface.
- Changing `--format` semantics beyond suppressing hints on json.

## Acceptance

- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [ ] Dogfooding pass: run the five hinted commands and confirm the tips
      are accurate and actionable.
- [ ] First-time reader can find install + quick start on the README in
      under 30 seconds.

## Related

- [[../backlog/drill-down-hints]]
- [[../backlog/lean-readme]]
- hyalo iter-107 commit `d28325d` (drill-down hints reference)
- hyalo commit `4b6df49` (lean README reference)
