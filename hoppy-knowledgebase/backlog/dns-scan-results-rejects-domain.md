---
title: "`dns zone scan results` rejects `--domain`, breaking the pre-zone scan workflow"
type: backlog
date: 2026-05-31
status: resolved
priority: medium
origin: dogfooding-2026-05-31
tags:
  - cli
  - dns
  - scan
  - dx
  - workflow-break
---

# Scan-before-create workflow has no `results` step

`hoppy dns zone scan start` accepts **either** `--id <zone-id>` (for an
existing zone) **or** `--domain <domain>` (for a domain you don't yet
manage). It returns a `JobId`:

```sh
$ hoppy dns zone scan start --domain ractive.ch --format json
{
  "JobId": "edc95e69-…",
  "Status": 0
}
```

But the matching `results` command only takes `--id`:

```sh
$ hoppy dns zone scan results --domain ractive.ch
error: unexpected argument '--domain' found

Usage: hoppy dns zone scan results [OPTIONS] --id <ID>
```

So the canonical "scan a domain before onboarding it" workflow has no
follow-up — there is no zone to pass `--id`, and the `JobId` from `start`
isn't accepted by `results` either.

## Repro

```sh
hoppy dns zone scan start   --domain ractive.ch   # → JobId, no zone
hoppy dns zone scan results --domain ractive.ch   # → error
hoppy dns zone scan results --job-id <uuid>       # → unknown flag
```

## Suggested fix

Two options, either is fine:

1. **Mirror `start`'s flag shape.** Let `results` accept `--id <zone-id>`
   OR `--domain <domain>` (same `clap::ArgGroup`).
2. **Accept the job ID.** Add `--job-id <uuid>` to `results` so the user
   can carry the handle from `start` to `results` directly. This also
   future-proofs the "I started two scans on the same zone" case.

Option 1 has the smallest API surface and matches what users will
already try by symmetry with `start`.

## Related

- [[dns-scan-results-null-records]] — separate finding on the results
  payload shape
- [[live-dns-scan-flake]] — timing-related flake when results race the
  scan
