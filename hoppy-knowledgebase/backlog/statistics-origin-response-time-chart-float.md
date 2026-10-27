---
title: hoppy statistics fails because OriginResponseTimeChart values are floats
type: backlog
date: 2026-10-27
status: resolved
priority: high
origin: demo-prep-2026-10-27
resolved-in: iter-91
---

# `hoppy statistics` fails: `OriginResponseTimeChart` values are floats

`hoppy statistics` (account-level, `GET /statistics`) fails on the private account
every time:

```text
Error: failed to deserialise response body: invalid type: floating point `101.91867479402572`, expected i64 at line 1 column 163
```

`hoppy --debug statistics` shows the response is `200 OK` and the bad value is the
first entry of `OriginResponseTimeChart`:

```json
"AverageOriginResponseTime": 196,
"OriginResponseTimeChart": {
  "2026-09-27T00:00:00Z": 101.91867479402572,
  "2026-09-28T00:00:00Z": 125.83036168412676,
```

`AccountStatistics` in `crates/bunny-net-api/src/core/types.rs` types it as
`Option<BTreeMap<String, i64>>`. A daily average response time is naturally a
float, so the fixture probably had only whole-number values.

## Why it matters

The whole command is unusable as soon as the account has any origin traffic. It
is the obvious first command to show in a demo.

## Fix sketch

- [x] Make `origin_response_time_chart` a `BTreeMap<String, f64>`. Check every other
  `*_chart` map and `average_origin_response_time` against the OpenAPI spec and a
  live response, because any average or ratio can be fractional.
- [x] Check the per-zone statistics types (`pull-zone statistics`, `script statistics`,
  `container app statistics`) for the same i64-vs-float pattern.
- [x] Add a fractional value to the statistics fixture, inside an iteration that changes the
  type, the fixture and the tests together (fixture contract, decision 2026-07-10).
- [ ] Consider a lenient number deserializer for chart maps, so one odd value no
  longer fails the whole response.
