---
title: Out-of-range integers on u8 repr enums still fail the whole response
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: review-pr-106
tags:
  - api-drift
  - enums
---

# Out-of-range integers on u8 repr enums fail the whole response

`forward_compat_repr_enum!` (`crates/bunny-net-api/src/macros.rs`) keeps unknown
values as `Unknown(n)` so a new enum value no longer fails a list or get
response. The documented limit: a value that does not fit the repr type (for
example `300` or `-1` on a `u8` enum) still fails deserialization of the
containing struct, and so of the whole response.

Unlikely in practice (the API uses small codes), hence low priority.

- [ ] Decide: widen the stored value (e.g. `i64` in `Unknown`) or deserialize via `i64` and clamp/flag
- [ ] Add a unit test pinning the chosen behaviour for `256` and `-1`
