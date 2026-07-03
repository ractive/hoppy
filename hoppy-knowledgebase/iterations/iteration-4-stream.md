---
title: "Iteration 4 — Stream (Video)"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - stream
  - video
status: completed
branch: iter-4/stream
---

# Iteration 4 — Stream (Video)

**Goal:** Manage video libraries and videos — exercises the Stream API (different base URL and API key).

- [x] Stream library commands:
  - [x] `stream library list|get|create|update|delete`
- [x] Stream video commands:
  - [x] `stream video list --library-id <id>`
  - [x] `stream video get --library-id <id> --video-id <id>`
  - [x] `stream video upload --library-id <id> --file <path>`
  - [x] `stream video delete --library-id <id> --video-id <id> [--yes]`
- [x] Handle stream API key (`BUNNY_STREAM_KEY` or derived from library)
- [x] Video upload with progress bar — done in iter 7

**Deliverable:** Upload and manage videos via CLI.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-3-dns]] — previous iteration
- [[api/bunny-stream-api-research]] — Stream API research
- [[api/bunny-api-quirks]] — Stream-specific API quirks
- [[decision-log]] — Stream API key resolution, PascalCase quirk
