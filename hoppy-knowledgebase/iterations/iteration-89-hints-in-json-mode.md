---
title: Iter-89 — stop suppressing drill-down hints under --format json
type: iteration
date: 2026-10-23
tags:
  - iteration
  - dx
  - llm
status: completed
branch: iter-89/hints-in-json-mode
---

# Iter-89 — hints in JSON mode

Supersedes PR #99 (`iter-86/hints-in-json-mode`), which conflicted after
iter-88 centralised hint command lines in `hint_cmds.rs`. Same intent,
re-implemented on current main.

## Why

`--format json` implied `--no-hints` (`main.rs`), so the one mode agents use
had zero next-step guidance — contradicting the LLM rationale in
[[backlog/drill-down-hints]]. Hints belong on stderr in every format, never
inside the JSON payload (an envelope field would break `jq '.[]'` and be
stripped by filters).

## Design

- Remove the `!matches!(cli.format, Json)` clause from `hints_enabled`;
  `--no-hints` / `--quiet` remain the opt-outs.
- Move format-agnostic drill-down tips out of non-JSON branches: dns zone
  scan start, search pagination, country list, billing payment-requests,
  stream library list, audit-log continuation. (pull-zone list and
  container create were already format-agnostic on main.) Command lines
  come from `hint_cmds.rs`.
- Format-*specific* tips stay table-only: truncation tips ("use --format
  json for full values") and the statistics `--hourly` table-view tip would
  be self-referential noise under JSON.

## Tasks

- [x] Remove JSON clause from `hints_enabled` in `main.rs`
- [x] Ungate drill-down tips (dns scan start, account x4, stream)
- [x] Flip coupled e2e tests; assert stdout stays pure JSON while hints print on stderr
- [x] Update `--no-hints` help text, MANUAL, help snapshot, CHANGELOG
- [x] fmt / clippy / test gates
- [x] Read-only dogfood: tip on stderr, `jq length` on stdout
- [x] PR, reviewed

## Deferred

- None; the `--reveal` tips were hoisted in review.
