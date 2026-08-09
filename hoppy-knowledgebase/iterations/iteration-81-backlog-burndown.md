---
title: Iter-81 — Backlog burn-down + dogfood round
type: iteration
date: 2026-08-09
tags: [iteration, backlog, dogfooding, testing]
status: in-progress
branch: iter-81/backlog-burndown
---

# Iter-81 — Backlog burn-down + dogfood round

## Why

Post iter-80 all planned API-coverage work is merged and the backlog held
8 `status: open` items of unknown freshness. This iteration triages every
open item against the current code, fixes what is fixable, documents what
is blocked upstream, and runs a dogfood round against the test account
(TEST_BUNNY_API_KEY) to live-verify the conclusions.

## Tasks

- [x] Prune stale "Planned Iterations (API Coverage)" section from
  [[development-roadmap]] (iters 13–18 completed long ago); add pointer to
  [[research/api-coverage-gap-analysis-2026-07]]
- [x] Triage all 8 open backlog items against current code (agent-verified
  with file:line evidence)
- [x] Close stale: [[backlog/pull-zone-log-forwarding-fields-missing]]
  (all 7 flags exist since coverage wave) and
  [[backlog/sz-create-json-password-string-literal]] (create re-fetches
  real credentials; live-verified)
- [x] Close by decision: [[backlog/fixture-git-history-dead-deployment-keys]]
  (accepted — keys dead, history rewrite not worth it) and
  [[backlog/json-output-casing-inconsistency]] (documented as contract in
  `docs/MANUAL.md`; NDJSON `--flat` filed as future roadmap item)
- [x] Implement real `dogfooding/cleanup.sh` (closed 9-prefix allowlist,
  narrow-only `--prefix`, dry-run default, cascade-first ordering,
  failure-tolerant; shellcheck-clean, offline-tested against a stub)
- [x] Generalize live-test retry-on-401 into
  `support::hoppy_live_json_with_401_retry`; wrap stream library
  statistics + all five collection Stream-API calls; leave core-API steps
  bare ([[backlog/live-stream-statistics-401-retry]])
- [x] Fix container `-upd` leak: cleanup guard now registers
  `container app delete --id <id> --cascade` (root cause was the cascade
  refusal on the auto-managed pull zone, not the rename)
- [x] Dogfood round (see below)
- [x] Round 2 — fix `--debug` request-body omission across ALL non-core
  clients (containers/stream/database/compute/storage/shield; shared
  `recording/debug.rs` helper; response bodies now redacted too — shield
  and friends were leaking secrets into `<<<` output; reveal flag plumbed
  through ~35 handler signatures; 2 new e2e tests)
  ([[backlog/mc-debug-omits-request-body]])
- [x] Round 2 — live-settle db fork `--group`: API ignores it (fork into
  empty group hit source-namespace quota while direct create into the
  same group succeeded) → field + flag removed, pinned by
  `db_fork_rejects_group` ([[backlog/db-fork-group-field-drift]])
- [x] Round 2 — SOLVED the 3-month log-forwarding empty-400: the API
  requires `token` despite the spec marking it optional (A/B on same app:
  tokenless → 400, with token → 201). `container logs` now auto-generates
  a session token; `--token` required on create/update; quirk + no-app-id
  validation recorded in [[api/bunny-api-quirks]]
  ([[backlog/log-forwarding-create-empty-400]])
- [x] Round 2 — new friction filed: [[backlog/db-create-slug-length-mismatch]]
  (local 24-char validation doesn't prevent upstream 500 at 19 chars),
  [[backlog/db-group-create-region-vocab]] (two region vocabularies,
  unvalidated)
- [x] Live-verify fixes: `live_stream_library_lifecycle`,
  `live_stream_collection_lifecycle`, `live_container_app_lifecycle`
  green + no new leaks after run
- [x] Quality gates (fmt, clippy -D warnings, test --quiet), PR

## Dogfood round (2026-08-09)

- `auth check` OK; leak census: 1 container app
  (`hpmc-1783703603606-0-upd` — the reproducible cascade leak), 15 inert
  orphaned shield zones, all other surfaces clean.
- `cleanup.sh` dry-run matched exactly the one leak; `--yes` deleted it.
- Storage-zone create password fix live-verified (real 42-char password
  in create JSON; zones deleted afterwards).
- Container log-forwarding create still 400/empty upstream → documented
  as known-broken in `docs/MANUAL.md`;
  [[backlog/log-forwarding-create-empty-400]] stays open (needs a
  bunny.net ticket). Its `get`-returns-404 side issue is fixed (returns
  `null` now).
- New regression found: mc client `--debug` omits request bodies →
  [[backlog/mc-debug-omits-request-body]] (open, low).
- db fork `--group` drift not testable (no dbs/groups on test account;
  PITR forks need aged snapshots) →
  [[backlog/db-fork-group-field-drift]] annotated, stays open.

## Acceptance

- [x] All 8 previously-open backlog items either resolved with evidence
  or annotated with a concrete path forward
- [x] Test account clean after full round (cleanup.sh dry-run: 0 matches)
- [ ] Gates green, PR merged

## Related

- [[development-roadmap]]
- [[dogfooding/dogfooding-playbook]]
- [[research/api-coverage-gap-analysis-2026-07]]
