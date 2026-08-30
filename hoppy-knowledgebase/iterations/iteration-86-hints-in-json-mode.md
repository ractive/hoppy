---
title: Iter-86 — stop suppressing drill-down hints under --format json
type: iteration
date: 2026-08-30
tags:
  - iteration
  - dx
  - llm
status: in-progress
branch: iter-86/hints-in-json-mode
---

# Iter-86 — hints in JSON mode

## Why

`--format json` implied `--no-hints` (`main.rs`), so the one mode agents use
had zero next-step guidance — contradicting the LLM rationale in
[[backlog/drill-down-hints]]. Per the AXI gap analysis
([[research/axi-agent-experience-interface-2026-08-29]]) and discussion
(2026-08-29): hints belong on stderr in every format, never inside the JSON
payload (an envelope field would break `jq '.[]'` and be stripped by filters).

## Design

- Remove the `!matches!(cli.format, Json)` clause from `hints_enabled`;
  `--no-hints` / `--quiet` remain the opt-outs.
- Move format-agnostic drill-down tips out of non-JSON branches at call
  sites that had them table-gated: pull-zone list, dns zone scan start,
  search pagination, country list, stream library list, billing
  payment-requests (the last caught in review).
- Format-*specific* tips stay gated where they are: truncation tips
  ("use --format json for full values") and the statistics `--hourly`
  table-view tip would be self-referential noise under JSON.

## Tasks

- [x] Remove JSON clause from `hints_enabled` in `main.rs`
- [x] Ungate drill-down tips (pull_zone, dns, account x2, stream)
- [x] Flip coupled e2e tests; assert stdout stays pure JSON while hints print on stderr
- [x] Update `--no-hints` help text, MANUAL, help snapshot
- [x] fmt / clippy / test gates
- [x] PR — #99, reviewed (local + Copilot), findings fixed

## Deferred

- `--reveal` tips (apikey/billing) still table-gated although JSON output is
  redacted too — candidate follow-up, low impact.
