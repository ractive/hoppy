---
title: Iter-58 — DNS scan results Domain column
type: iteration
date: 2026-06-01
tags:
  - iteration
  - dns
  - scan
  - polish
status: in-progress
branch: iter-58/dns-scan-domain-column
---

# Iter-58 — DNS scan results Domain column

## Why

After [[iteration-53-dns-scan-results-by-domain]] added `--domain`
to `dns zone scan results`, the rendered `Domain` column is still
blank (`-`). We already know the domain — we used it to resolve
the zone id — so filling the column client-side is a free DX win.

See [[../backlog/dns-scan-results-domain-column-empty]].

## Scope

### 1. Plumb the resolved domain [2/2]

- [x] In the `--domain` code path, capture the user-supplied
      domain and pass it to the render layer alongside the scan
      result.
- [x] In the `--id` code path, look up the domain via the zone
      list (one extra read) so both paths render symmetrically.

### 2. Render [2/2]

- [x] Populate the `Domain` column in the table output.
- [x] Include the `Domain` field in `--format json` and `--format
      text`. Decide whether to add it to the response model
      (preferred) or merge in the renderer.

### 3. Tests [2/2]

- [x] E2E mock test that `scan results --domain X` shows `X` in
      the Domain column.
- [x] E2E mock test that `scan results --id <z>` also shows the
      resolved domain.

## Out of scope

- Caching the zone-list lookup across calls.
- Touching `scan start` output (already prints the next-command
  hint).

## Acceptance Criteria

- [x] `hoppy dns zone scan results --domain X` shows `X` in the
      Domain column.
- [x] `hoppy dns zone scan results --id <z>` shows the resolved
      domain in the Domain column.
- [x] `--format json` includes the `Domain` field.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/dns-scan-results-domain-column-empty]]
- [[../iterations/iteration-53-dns-scan-results-by-domain]]
- [[../dogfooding/session-2026-06-01]]
