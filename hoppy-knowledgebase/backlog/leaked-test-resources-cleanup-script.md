---
title: cleanup.sh still a skeleton — 19 leaked resources observed
type: backlog
date: 2026-05-15
status: resolved
tags:
  - dogfooding
  - cleanup
  - leaks
  - tests
---

# Live-api test runs leak resources, cleanup.sh doesn't clean them

## State on 2026-07-10 (fixture-refresh pre-flight scan)

Still leaking. Pre-flight enumeration before the fixture-refresh sweep found
10 leftovers (some from May, some from the 2026-07-10 iter-66..77 live run):

| Surface        | Leaks | IDs |
|----------------|-------|-----|
| pull zones     | 2     | 5857625 (`hoppy-edge-rule-*`), 5857634 (`hoppy-shield-test-*`) |
| storage zones  | 1     | 1515810 (`hpst-1778785767589-1`) |
| container apps | 9     | `hpmc-*-upd` — AGgJrAa8x91qbOB, AhURpuhAmIjaJpP, CBI5MrzQsZLio4b, I8jMdqTeJGAFFl1, Iw2g5HcXM7k407G, Y8OkKr3R0iNtd3E, eWgSqyF6FGqEI4m, geGegD8mXdxMLng, iIzjD0NJ52AoDbE |
| shield zones   | 1     | 118829 — orphan; its pull zone 5830062 is gone and the CLI has no `shield zone delete` |

The container-app leaks are all `-upd` suffixed, and the two fixture-refresh
sweeps on 2026-07-10 each leaked **exactly one** new `-upd` app
(Iw2g5HcXM7k407G, Y8OkKr3R0iNtd3E): the container update-lifecycle test's
rename step reliably breaks the cleanup stack's delete — this is a
reproducible bug, not an occasional flake.

**Cleanup executed 2026-07-10:** all pull zones, storage zones, and
container apps deleted (`container app delete --cascade --yes` handles the
auto-managed pull zones). What remains is 10 **orphaned shield zones**
(118829, 122317, 122354, 122393, 122412, 122444, 149999, 150002–150004),
each referencing an already-deleted pull zone. There is no way to remove
them: the public spec has no `DELETE /shield/shield-zone/{id}` and a
speculative call returns 405. They appear to be inert server-side residue —
shield-zone creation during tests should assume old orphans may be present
in `shield zone list` output.

## State on 2026-05-15

A fresh `hoppy <noun> list` against the test account showed leaks from
prior `cargo test --features live-api` runs:

| Surface       | Leaks | Pattern               |
|---------------|-------|-----------------------|
| pull zones    | 2     | `hoppy-edge-rule-*`, `hoppy-shield-test-*` |
| storage zones | 1     | `hpst-*`              |
| container apps| 13    | `hpmc-*`              |

`hoppy-knowledgebase/dogfooding/cleanup.sh` is still the skeleton
described in `iteration-25-publish.md` — each surface block just prints
"not yet implemented".

## Why this hurts

- Container apps cost real money per hour even idle. 13 leaked apps
  multiplied by even a low hourly rate is a real bill on the test
  account.
- The leaks pollute every dogfooding `hoppy <noun> list` smoke test,
  making it harder to spot the resources from the current session.
- The recent live-api test improvements (shape-first asserts) make
  leaks easier — tests are more tolerant, so a panic-cleanup miss is
  less likely to be caught.

## Want

- Real `cleanup.sh --yes` that deletes everything matching `hoppy-test-`
  / `hpmc-` / `hpst-` / `hoppy-edge-rule-` / `hoppy-shield-test-`.
- `--cascade` for container apps (already supported by `container app
  delete`) so auto-managed pull zones get cleaned too.
- A guardrail that refuses to delete anything not matching one of a
  closed set of test prefixes.

## Out of scope (separate item)

- Why the test runs are leaking in the first place — the panic-safe
  `run_lifecycle()` stack should already clean up. Possibly a panic
  during cleanup is escaping. Worth investigating after cleanup.sh is
  real (so the leaks stop piling up and the leaks-per-run counter is
  observable).

## Resolution (2026-08-09, iter-81)

`cleanup.sh` is now a real implementation (356 lines, bash 3.2
compatible, shellcheck-clean):

- Closed 9-prefix allowlist (`hoppy-test-`, `hoppytest-`,
  `hoppy-edge-rule-`, `hoppy-shield-test-`, `hpmc-`, `hpst-`, `hpsc-`,
  `hpscs-`, `hpscv-`); `--prefix=` can only narrow, never widen.
- Dry-run by default; `--yes` deletes. Container apps go first with
  `--cascade`, then stream libraries, scripts, pull zones, storage
  zones, DNS zones. Shield zones are report-only (no delete API).
- Per-resource failure collection — one failed delete doesn't abort the
  sweep; exit 1 with a summary.

First live run (2026-08-09): dry-run matched exactly one leak
(`hpmc-1783703603606-0-upd`), `--yes` deleted it. Account is clean apart
from 15 inert orphaned shield zones.

The "why do runs leak" question from *Out of scope* is answered: the
container lifecycle test registered its cleanup delete without
`--cascade`, so `handle_app_delete` refused once the endpoint step had
provisioned an auto-managed pull zone, and the best-effort cleanup stack
swallowed the refusal. Fixed in the same iteration (cleanup guard now
passes `--cascade`).
