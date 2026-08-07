---
title: "Iteration 1 — First Service: Pull Zones (Core API)"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - core-api
  - first-service
  - pull-zone
status: completed
branch: iter-1/pull-zones
---

# Iteration 1 — First Service: Pull Zones (Core API)

**Goal:** Full CRUD for pull zones against the live API. This proves out the entire vertical stack.

- [x] HTTP client setup (hand-written reqwest — codegen abandoned in iter 0.5)
- [x] Shared request/response plumbing: base URL, auth, error mapping
- [x] Debug logging of HTTP requests (`--debug` flag)
- [x] Pull Zone commands:
  - [x] `list` — paginated listing with `--search`, `--page`, `--per-page`
  - [x] `get --id <id>` — single pull zone details
  - [x] `create --name <name> --origin-url <url> [options]` — create pull zone
  - [x] `update --id <id> [options]` — update pull zone settings
  - [x] `delete --id <id> [--yes]` — delete with confirmation prompt
  - [x] `purge --id <id> [--cache-tag <tag>]` — purge cache (by tag or all)
- [x] Table output: pick sensible default columns (id, name, origin URL, status)
- [x] Pagination: `--page`, `--per-page` flags
- [x] Integration test: at least one test that mocks the API response (deferred — need real API responses as fixtures first)

**Deliverable:** `BUNNY_API_KEY=xxx hoppy pull-zone list --format json` returns real data.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-1-code-review]] — code review findings from this iteration
- [[iterations/iteration-0.5-codegen-experiment]] — codegen experiment that preceded this
- [[api/bunny-api-client-patterns]] — patterns established during this iteration
