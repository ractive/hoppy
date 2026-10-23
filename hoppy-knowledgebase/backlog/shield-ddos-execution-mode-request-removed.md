---
title: --ddos-execution-mode targets a request field the spec removed
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - shield
  - ddos
  - api-drift
  - dogfooding
---

# `ShieldZoneRequest.dDoSExecutionMode` removed from the spec

The fresh shield spec drops `dDoSExecutionMode` from `ShieldZoneRequest`. It is
still present on `ShieldZoneResponse`. hoppy still serializes it from
`shield zone update --ddos-execution-mode`
(`crates/bunny-net-api/src/shield/types.rs` `ShieldZoneRequest.d_do_s_execution_mode`,
`crates/hoppy-cli/src/commands/shield.rs:770`).

**Impact: unknown.** Either the API silently ignores it, in which case the
flag is a no-op, or it rejects it.

- [ ] Dogfood: update a zone with only `--ddos-execution-mode` and re-read the zone
- [ ] If it is ignored or rejected: deprecate the flag and drop the field from the request, keeping it on the response

See [[research/openapi-drift-2026-10-23]].
