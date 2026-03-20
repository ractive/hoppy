---
title: "Iteration 2 — Storage Zones + File Operations"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - storage
  - file-operations
status: completed
branch: iter-2/storage
---

# Iteration 2 — Storage Zones + File Operations

**Goal:** Manage storage zones and upload/download files — this exercises the Storage API (different base URL, different auth key).

- [x] Storage Zone commands (Core API):
  - [x] `storage-zone list|get|create|update|delete`
- [x] Storage file commands (Storage API — different base URL):
  - [x] `storage upload --zone <name> --remote-path <path> --file <local-path>`
  - [x] `storage download --zone <name> --remote-path <path> [--output <local-path>]`
  - [x] `storage ls --zone <name> [--path <dir>]`
  - [x] `storage rm --zone <name> --remote-path <path> [--yes]`
- [x] Handle per-zone storage API key (from zone details or `BUNNY_STORAGE_KEY` env var)
- [x] Progress bar for upload/download (stderr, only if TTY) — done in iter 7
- [x] JSON list output should include pagination envelope (`current_page`, `total_items`, `has_more_items`), not just the items array — apply consistently across all list commands including pull zones
- [x] Integration tests with mock HTTP server (carried from iter 1 — record real API responses as fixtures first)
- [x] Consolidate duplicate `PaginatedList` and `ApiError` types across `bunny-api-core` and `bunny-api-compute` — investigated, intentionally kept separate with documentation (crates are independent, no shared dependency warranted)

**Deliverable:** Upload and download files to/from bunny.net storage.

## Related
- [[development-roadmap]] — project roadmap
- [[iterations/iteration-1-pull-zones]] — previous iteration
- [[decision-log]] — storage auth resolution decision
