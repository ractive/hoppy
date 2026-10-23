---
title: --ddos-execution-mode targets a request field the spec removed
type: backlog
date: 2026-10-23
status: completed
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - shield
  - ddos
  - api-drift
  - dogfooding
resolved-in: iter-87
---

# `ShieldZoneRequest.dDoSExecutionMode` removed from the spec

The fresh shield spec drops `dDoSExecutionMode` from `ShieldZoneRequest`. It is
still present on `ShieldZoneResponse`. hoppy still serializes it from
`shield zone update --ddos-execution-mode`
(`crates/bunny-net-api/src/shield/types.rs` `ShieldZoneRequest.d_do_s_execution_mode`,
`crates/hoppy-cli/src/commands/shield.rs:770`).

**Impact: unknown.** Either the API silently ignores it, in which case the
flag is a no-op, or it rejects it.

- [x] Dogfood: update a zone with only `--ddos-execution-mode` and re-read the zone
- [x] If it is ignored or rejected: deprecate the flag and drop the field from the request, keeping it on the response

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

Live check (2026-10-23, throwaway `hoppy-test-iter87-ddos` zone, Basic plan): `shield zone update --ddos-execution-mode 0` left the zone at 1 on re-read, so the API ignores it. The field was dropped from `ShieldZoneRequest` (kept on the response) and the flag is hidden, deprecated and warns. Not verified on a higher plan. The zone value was left as found (1); the pull zone was deleted via the cleanup script (the shield zone itself cannot be deleted).

See [[iterations/iteration-87-openapi-drift-fixes]].
