---
title: "Iteration 6 — Edge Scripting + Magic Containers"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - scripting
  - containers
  - compute
status: completed
branch: iter-6/scripting-containers
---

# Iteration 6 — Edge Scripting + Magic Containers

**Goal:** Manage serverless scripts and containers.

- [x] Edge scripting commands:
  - [x] `script list|get|create|update|delete` with full options (pagination, search, linked pull zones)
  - [x] `script publish` (replaces `deploy` — matches API endpoint name)
  - [x] `script code get|update` (update supports `--code` inline or `--file` path)
  - [x] `script release list|get-active` with pagination
  - [x] `script variable list|add|update|delete`
  - [x] `script secret list|add|update|delete`
  - [x] `script statistics` with `--date-from`, `--date-to`, `--hourly`
- [x] Debug mode support (`--debug` flag) added to ComputeClient
- [x] Confirmation prompts for destructive operations (`--yes` to skip)
- [x] `deployment_key` excluded from JSON output (`#[serde(skip_serializing)]`)
- [x] 28 wiremock integration tests with fixture-based responses
- [x] Error handling tests (401 unauthorized, 404 not found)
- [x] Request body validation in tests (body_json matchers)
- [x] Magic container API client (`bunny-api-containers` crate) — hand-written from docs (no OpenAPI spec available)
  - [x] 47 endpoints across 11 resource groups (applications, containers, registries, endpoints, volumes, autoscaling, regions, nodes, pods, limits, log forwarding)
  - [x] Full type coverage: all request/response structs, enums, cursor-based pagination
  - [x] Error handling via `ProblemDetails` + `ErrorDetails` (RFC 7807 pattern, like Shield)
  - [x] 13 unit tests (serde roundtrip, client construction, auth header)
  - [x] 57 wiremock integration tests with real API fixtures (all 47 endpoints + error handling + debug mode)
  - [x] Fix enum serde casing: API returns camelCase, added `rename_all = "camelCase"` to 10 enums
  - [x] CLI commands for Magic Containers — full `container` command tree wired (apps, templates, endpoints, volumes, registries, regions, nodes, pods, limits, log forwarding)

**Deliverable:** Deploy and manage edge scripts. Magic Containers API client and CLI commands fully implemented.

## Related
- [[development-roadmap]] — project roadmap
- [[iterations/iteration-5-shield]] — previous iteration
- [[api/magic-containers/magic-containers-applications-api]] — Magic Containers API reference
- [[decision-log]] — Deploy renamed to Publish, Magic Containers hand-written from docs
