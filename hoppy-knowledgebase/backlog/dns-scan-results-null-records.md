---
title: "dns zone scan results panics on `Records: null` (in-progress scan)"
type: backlog
date: 2026-05-10
status: completed
priority: high
origin: dogfooding-2026-05-10
---

# `dns zone scan results` errors when the scan is still pending

Repro:

```sh
hoppy dns zone scan start --id <zone-id>      # status=Pending
hoppy dns zone scan results --id <zone-id>    # immediately
# Error: failed to deserialise response body:
# invalid type: null, expected a sequence at line 1 column 187

```

The bunny.net API returns `"Records": null` while `Status` is `Pending` /
`Running`. The hoppy deserialiser declares the field as `Vec<…>` (non-optional)
so serde fails. Once the job completes, `Records` becomes `[]` and the
deserialise succeeds — that's why the second call (a few seconds later)
worked during dogfooding.

## Fix

Make the records field tolerate null, e.g.:

- `Option<Vec<Record>>` with `serde(default)`, or
- `Vec<Record>` with a `deserialize_with` helper that turns null into an
  empty vec (similar to `deserialize_string_lossy_option` in
  `crates/bunny-api-core/src/serde_helpers.rs`).

Iter-17 surface — this is the brand-new scan API. Should add a unit test
with a fixture body of the in-progress shape.
