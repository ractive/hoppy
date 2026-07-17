---
type: adr
title: Consolidate bunny-net-api-* into one bunny-net-api crate
status: accepted
date: 2026-05-10
deciders:
  - hoppy-maintainers
---

# Consolidate bunny-net-api-* into one bunny-net-api crate

## Context and Problem Statement

The workspace shipped eight per-service `bunny-net-api-*` crates. Every release
bumped all eight in lockstep and cross-cutting changes had to be repeated per
crate. Was the split earning its keep?

## Considered Options

- Keep the eight per-service crates
- Consolidate into one `bunny-net-api` crate with feature-gated modules

## Decision Outcome

Chosen option: consolidate (iter-32). No downstream consumer exists, so the
split was premature. One `bunny-net-api` library with feature-gated modules
mirrors hyalo's shape. The CLI moves to a `hoppy-cli` package (binary still
`hoppy`); install via `cargo install hoppy-cli`. This supersedes the earlier
decision to keep `PaginatedList`/`ApiError` duplicated across separate
per-service crates.
