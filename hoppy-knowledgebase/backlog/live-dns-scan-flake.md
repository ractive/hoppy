---
title: "live_dns_zone_record_scan_lifecycle times out waiting for scan to finish"
type: backlog
date: 2026-05-14
status: planned
priority: medium
origin: dogfooding-2026-05-14 (iter-33 §5 sweep)
---

# DNS scan lifecycle live test flakes

In the 2026-05-14 live-api sweep (test account), the scan lifecycle test
panicked:

```
thread 'cli_dns::live_dns_zone_record_scan_lifecycle' panicked at
crates/hoppy-cli/tests/e2e/cli_dns.rs:642:9:
scan did not reach a terminal state (last status: Some(0))
```

The test polls `hoppy dns zone scan results --id <zone>` until a terminal
status, but in this run the scan stayed in `Status=0` (Pending) past the
poll budget.

## Likely causes

- Poll budget too tight for a real scan run — scans can sit Pending for
  longer than a fixed local timeout.
- The zone created by the test had no resolvable records to scan, so the
  upstream scan never advanced — needs a sanity check on what the test
  zone actually contains.

## Fix

- Either extend the poll budget meaningfully (and call it out in a
  comment), or
- Switch the assertion to "scan started + status field is well-typed" and
  drop the terminal-state requirement — terminal-state behaviour is the
  API's, not hoppy's.
