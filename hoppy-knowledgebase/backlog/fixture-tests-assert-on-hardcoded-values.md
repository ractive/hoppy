---
title: wiremock tests assert on hand-authored fixture values, breaking live-refresh
type: backlog
date: 2026-05-14
status: completed
priority: high
origin: dogfooding-2026-05-14 (iter-34 §5 dogfooding round)
---

# Tests are coupled to specific fixture values, not response shapes

The iter-34 fixture-refresh tool works end-to-end: it maps recordings to
the descriptive-name fixtures and overwrites the ones that drifted. The
2026-05-14 dogfooding sweep produced 14 drifted fixtures across `core/`
and `containers/`. Applying them, however, breaks **7 offline tests**.

## Why

Wiremock tests assert on specific values that the fixture was hand-authored
to contain, not on the response shape:

```rust
// crates/bunny-net-api/tests/core/e2e/billing_api.rs
assert!((billing.balance - 42.50).abs() < f64::EPSILON);
```

`billing_get.json` is hand-authored with `Balance: 42.50`. A live recording
from the test account has `Balance: 0` — the assertion blows up.

Other concrete failures from the sweep:

- `pullzone_api::get_pull_zone_returns_single_zone` — asserts `id == 1001`,
  recording has `id == 5856347` (real test-account pull zone ID).
- `statistics_api::get_account_statistics_returns_data` — asserts on
  specific bandwidth values present in the hand-authored fixture.
- `dns_api::list_dns_zones_*` — assert on the IDs / counts in the
  paginated fixture.

## Implication

The iter-34 refresh tool **cannot be used as an automated refresh** while
tests are written this way. Two paths forward, neither in iter-34's scope:

1. **Rewrite tests to assert on shape, not values.** Verify field types,
   serde round-trip, default fallbacks, and key invariants (e.g. "balance
   is non-negative", "list is non-empty when API returned items"). Drop
   `== 42.50` style asserts. This is a long tail of small per-test
   rewrites.

2. **Keep tests strict and hand-curate refreshes.** Treat live recordings
   as a *prompt* for a human to look at: "field X appeared in the live
   response but isn't in the fixture — should we add it and update the
   assert?". The refresh tool would still produce the drift report, but
   `--apply` becomes ill-defined.

(1) is the right long-term answer. (2) is what we're effectively doing
today and is fine for now.

## Suggested next step

File this iteration's drift report, the 7 failing tests, and the 14
recording diffs as a starting point. Sample 2–3 tests, rewrite them to
shape-asserts, confirm the refresh tool's `--apply` no longer breaks
them. If the pattern works, file a follow-up iter to rewrite the rest
in batches.

## Related

- [[iterations/iteration-34-fixture-mapper]] — the tool that surfaced
  this.
- [[iterations/iteration-33-fixture-refresh]] — the iter that shipped
  the recording plumbing.
- [[fixture-recording-name-mismatch]] — the iter-34-precursor problem,
  now fixed.
