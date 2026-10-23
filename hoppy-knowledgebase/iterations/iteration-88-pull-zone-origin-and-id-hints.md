---
title: Iter-88 - pull-zone Origin column and positional-id hints
type: iteration
date: 2026-10-23
tags:
  - iteration
  - dogfooding
  - pull-zone
  - hints
status: completed
branch: iter-88/pull-zone-origin-and-id-hints
---

# Iter-88 - pull-zone Origin column and positional-id hints

Backlog: [[backlog/pull-zone-list-origin-column-blank]], [[backlog/hints-suggest-id-flag-form]]. Follows [[iterations/iteration-86-dogfood-oct-fixes]]; remaining `--id`-only commands: [[backlog/id-arg-remaining-commands]].

## Tasks

### Origin column
- [x] `origin_label` derives the label from `OriginType` (StorageZone, MagicContainerEndpoint, OriginUrl, DnsAccelerate, LoadBalancer, Script, PushZone)
- [x] Storage zone names fetched with one paginated `list_storage_zones` only when storage-origin rows exist; fallback `storage zone <id>` plus a warning on failure
- [x] JSON output unchanged (raw API shape)
- [x] Unit tests for every label; e2e for table labels, lookup failure, JSON passthrough
- [x] `pull-zone get` checked: vertical table shows raw `OriginUrl`/`OriginType` fields, no `Origin` column, left as is

### Hints
- [x] All hint command lines centralised in `hint_cmds.rs`; positional form where the command accepts it
- [x] Test parses every hint with clap; `--id`-only hints pinned by a test that fails when they become positional
- [x] MANUAL.md updated

### Gates and dogfooding
- [x] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --quiet`
- [x] Read-only dogfood of `pull-zone list` and `pull-zone get` against the real account

## Acceptance criteria
- [x] `pull-zone list` never shows a blank or placeholder Origin
- [x] No hint suggests `--id` for a command that accepts the positional id
