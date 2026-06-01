---
title: statistics and stream library statistics return chart maps in arbitrary order
type: backlog
date: 2026-06-01
status: planned
priority: medium
origin: dogfooding-2026-06-01
tags:
  - cli
  - statistics
  - stream
  - json
  - determinism
---

# Chart maps come out unordered in JSON

`hoppy statistics --format json` and `hoppy stream library statistics
--id <id> --format json` return chart fields as maps keyed by ISO date:

```json
{
  "OriginResponseTimeChart": {
    "2026-05-05T00:00:00Z": 0,
    "2026-05-22T00:00:00Z": 0,
    "2026-05-30T00:00:00Z": 0,
    "2026-05-13T00:00:00Z": 0,
    ...
  }
}
```

The keys come out in HashMap insertion order, not chronological order.
This:

- breaks deterministic diffing of fixtures across runs (every refresh
  changes byte ordering even when values are identical)
- makes the human-eyeballable JSON output much harder to scan
- forces every downstream JSON consumer (jq, plotting, etc.) to re-sort
  before doing anything time-series-shaped

## Fix

In the deserialised model, store these charts as `BTreeMap<String, T>`
or sort the entries on serialise (custom `Serialize` impl, or just
collect into `Vec<(date, value)>` after sorting). Affected fields:

- `statistics`: `OriginResponseTimeChart`, plus any other `*Chart`
  maps on the GetStatistics response.
- `stream library statistics`: `viewsChart`, `watchTimeChart`,
  `countryViewsChart`, `countryWatchTimeChart`, plus any sibling maps.

Touch every `chart`-typed field uniformly so a future surface (db
live, container metrics) doesn't reintroduce the same drift.

## Acceptance

- Re-running `hoppy statistics --format json` produces byte-identical
  output between two runs against the same backend state.
- Chart keys appear in ascending date order.
- Existing fixtures regenerated and reviewed (no shape change, just
  ordering).
