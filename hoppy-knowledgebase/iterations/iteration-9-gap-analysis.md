---
title: "Iteration 9 — Gap Analysis & Missing CLI Commands"
type: iteration
date: 2026-03-19
tags:
  - iteration
  - gap-analysis
  - audit
status: completed
branch: iter-9/gap-analysis
---

# Iteration 9 — Gap Analysis & Missing CLI Commands

**Goal:** Audit all API client methods against wired CLI commands, wire any missing ones.

- [x] Stream video commands:
  - [x] `stream video update --library-id <id> --video-id <id> [--title <title>] [--collection-id <id>]`
  - [x] `stream video fetch --library-id <id> --url <url>` — ingest video from remote URL (async)
- [x] Stream collection commands:
  - [x] `stream collection list|get|create|update|delete --library-id <id>`
- [x] Edge scripting commands:
  - [x] `script rotate-deployment-key --id <id>` — rotate deployment key
- [x] URL query parameter redaction in `--debug` output (privacy)
- [x] v0.1.0 test plan document

**Deliverable:** All API client methods have corresponding CLI commands. No gaps between client crates and CLI layer.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-8-release]] — previous iteration
- [[testing/test-plan-v0.1.0]] — test plan created during this iteration
