---
title: Iter-86 - dogfooding fixes (endpoint id, positional id, Magic Containers zones)
type: iteration
date: 2026-10-23
tags:
  - iteration
  - dogfooding
  - pull-zone
  - containers
status: completed
branch: iter-86/dogfood-oct-fixes
---

# Iter-86 - dogfooding fixes

Backlog: [[backlog/container-get-json-drops-endpoint-id]], [[backlog/pull-zone-get-positional-id]], [[backlog/pull-zone-list-omits-magic-container-zones]]. Research: [[research/pull-zone-list-magic-containers-2026-10-23]]. Leftovers: [[backlog/id-arg-remaining-commands]].

## Tasks

### Endpoint id
- [x] `ContainerEndpoint.id` modelled (required, per current spec)
- [x] `container get` JSON keeps endpoint `id`; table lists template endpoints with their id
- [x] Unit + e2e tests

### Positional id
- [x] Reusable `IdArg<T>` (positional or `--id`, conflict and missing are usage errors)
- [x] Applied to `get` and all other plain-`id` subcommands
- [x] e2e per command group, both forms; clap debug-assert test
- [x] Docs: command-tree, MANUAL; remainder filed as backlog

### Magic Containers pull zones
- [x] Investigate list parameter (spec + live probe), record in research/
- [x] Fallback: apps -> endpoints -> get_pull_zone, dedupe, `App` column / `ContainerApp` JSON
- [x] `count` consistent; `--no-container-zones` opt-out; warning on lookup failure
- [x] Unit tests for merge/dedupe, e2e for list/count
- [x] `OriginType` gains DnsAccelerate (1) and PushZone (6)

### Sweeps
- [x] Missing-id sweep over all response types vs fixtures (added `PullZone.VideoLibraryId/DnsZoneId/DnsRecordId`)
- [x] Audit of other pull-zone enumerators (none besides list/count)

### Gates
- [x] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --quiet`
- [x] Release build dogfood (read-only)

## Acceptance criteria
- [x] `hoppy pull-zone get 123` and `--id 123` behave identically; both together or neither fail with exit 2
- [x] `container get --format json` shows each endpoint `id`
- [x] `pull-zone list`/`count` include container-owned zones, marked with their app
