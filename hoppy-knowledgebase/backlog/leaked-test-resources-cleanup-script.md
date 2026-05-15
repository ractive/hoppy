---
title: cleanup.sh still a skeleton — 19 leaked resources observed
type: backlog
date: 2026-05-15
status: open
tags:
  - dogfooding
  - cleanup
  - leaks
  - tests
---

# Live-api test runs leak resources, cleanup.sh doesn't clean them

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
