---
title: >-
  Iter-78 — repurpose fixture-refresh into a read-only API drift radar + sweep
  skill
type: iteration
date: 2026-07-10
tags:
  - iteration
  - fixtures
  - testing
  - dogfooding
  - live-api
  - tooling
status: completed
branch: iter-78/api-drift-radar
---

# Iter-78 — repurpose fixture-refresh into a read-only API drift radar + sweep skill

## Why

The 2026-07-10 fixture-refresh sweep ([[research/api-shape-drift-2026-07-10]])
proved that bulk-applying recorded live responses onto checked-in fixtures
fights the test suite by design: fixtures are **test contracts** whose values
the wiremock tests pin, so 36 of 38 applied files had to be reverted, and one
of the two survivors still needed a manual fix. Meanwhile the *diff* between
recordings and fixtures was highly valuable — it surfaced ~98 unmodeled video
library fields, DNS/shield gaps, stream casing drift, and four redaction
leaks.

Conclusion (agreed 2026-07-10): **observation never writes to the contract.**
The recording pipeline becomes a read-only drift radar; fixture updates only
happen inside deliberate iterations that change client types, fixtures, and
tests together.

## Target shape

```sh
# one command, run by the /api-drift-sweep skill after a recording run:
cargo run --bin fixture-refresh -- --recorded fixtures-recorded --shape-report
```

→ prints (and can write to a file) a per-domain markdown report:
new/removed key paths per endpoint (noise-filtered), unmapped recordings,
collisions, and a leak-audit section. Never modifies `fixtures/`.

## Scope

### 1. Remove `--apply` from fixture-refresh

- [x] Delete the `--apply` flag, the overwrite code path, and its tests.
- [x] Dry-run byte-drift listing (`drift: <fixture> (Δ N bytes)`) stays as
      informational output.
- [x] `unmapped:` / `collision:` listings stay (coverage + ambiguity signal).

### 2. Add `--shape-report` mode

- [x] For each mapped (recording, fixture) pair, diff **key paths + JSON
      types** (not values), in both directions (added vs removed keys).
- [x] Noise filters: ignore map keys that look like dates/timestamps
      (`01-07-2026`, `2026-07-10T…`) and array indices beyond `0`; keep the
      filter list in one place with unit tests.
- [x] Output: markdown grouped by domain → endpoint → added/removed key
      lists, suitable for pasting into a KB research note. `--out <file>`
      writes it directly.
- [x] Exit code signals "drift found" vs "clean" so the skill can branch.

### 3. Leak-audit section in the report

- [x] Scan recordings for: email-shaped values outside `example.com` /
      `<redacted>`; 72-char double-UUID account-key shapes; values under
      secret-ish key names (`*key*`, `*token*`, `*password*`, `*secret*`)
      that are not `<redacted>`/null/empty.
- [x] Optional account-specific literal patterns via a git-ignored file
      (e.g. `.hoppy-leak-patterns`, one regex per line) so real names/IDs
      never get hardcoded into the repo.
- [x] Any leak-audit hit is listed prominently and flips the exit code.

### 4. `/api-drift-sweep` skill

- [x] New project skill `.claude/skills/api-drift-sweep/SKILL.md` encoding
      the full procedure: pre-flight (**`TEST_BUNNY_API_KEY` only** — never
      any other key; `hoppy auth check`; read-only account leak scan),
      recording run (`HOPPY_RECORD_DIR` scratch dir, `--test-threads=1`,
      run in background), `--shape-report`, offline-suite verification
      (`cargo test --workspace --quiet` must stay green and `fixtures/`
      untouched), file the report as a dated KB research note, destroy the
      scratch dir, list account leftovers (deletion only with user
      approval; `container app delete --cascade`), end with ranked
      iteration candidates.
- [x] Skill explicitly states: never write to `fixtures/`, never commit
      recordings, never run with a non-test key.
- [x] Reference [[dogfooding/dogfooding-playbook]] for rationale and the
      known caveats (plan-tier error envelopes, `hpmc-*-upd` cleanup leak,
      orphaned shield zones).

### 5. Docs

- [x] Rewrite the playbook "Refreshing fixtures" section as "API drift
      radar": drop the apply step, describe report + iteration-lane update
      path, point to the skill.
- [x] Update `CLAUDE.md` Integration Tests note: `HOPPY_RECORD_DIR` feeds
      the drift radar; fixture updates happen only inside iterations.
- [x] `adding-a-feature.md` stays: `--record` is still the way to capture
      the raw payload when hand-crafting a **new** fixture.
- [x] Add the decision ("recordings never overwrite fixtures") to
      [[decision-log]].

### 6. Tests

- [x] Unit tests: shape-diff (added/removed/type-changed keys), noise
      filters, leak-audit patterns (incl. the double-UUID and email rules).
- [x] E2E: synthetic recorded-vs-fixture tree → assert report content and
      exit codes; assert `fixtures/` is never written.

## Out of scope

- Struct-level unknown-field detection (diff recordings against what the
  typed client re-serializes) — needs an endpoint→type mapping that doesn't
  exist in machine-readable form yet; file as backlog.
- Fixing the container update-lifecycle cleanup leak
  ([[backlog/leaked-test-resources-cleanup-script]]) — separate item.
- Implementing real `cleanup.sh` deletion logic — same backlog item.
- Recording non-2xx responses as error fixtures — possible follow-up.

## Risks and mitigations

- **Noise filters too aggressive** (hide real drift) or too loose (report
  unusable). Mitigation: filters unit-tested against the 2026-07-10 corpus
  captured in [[research/api-shape-drift-2026-07-10]].
- **Skill drifts from tooling flags.** Mitigation: skill invokes exactly one
  documented command per step; e2e test pins the CLI surface.

## Acceptance

- [x] `fixture-refresh` has no code path that writes under `fixtures/`.
- [x] `--shape-report` on a recorded sweep reproduces the substance of the
      2026-07-10 hand-written drift note (video library / DNS / shield /
      stream findings) with chart-date noise filtered out.
- [x] Leak audit flags a planted double-UUID and a planted email in a
      synthetic recording.
- [x] `/api-drift-sweep` skill file exists and walks the full procedure.
- [x] `cargo fmt` / `cargo clippy --workspace --all-targets -- -D warnings`
      / `cargo test --workspace --quiet` all clean.

## Outcome (dogfooded 2026-07-10)

Live sweep + `--shape-report` run against the test account: 98 recordings,
report reproduced every headline finding of the hand-written drift note
(video-library DRM fields, DNS acceleration/geo/routing, shield zone
config, pull-zone `CacheKeyHeaders`/`IpFamilyPolicy`, stream casing) with
zero date-noise lines. The leak audit paid for itself on its first run: it
caught `ZoneSecurityKey` (pull-zone URL-auth signing secret) escaping
redaction — fixed in `recording/redact.rs` (`securitykey` pattern) — and
two audit false positives (`PlayerKeyColor`, `PublicKey`) were added to the
exclusion list with tests. `fixtures/` verified byte-identical after the
full sweep; offline suite 1208/0.

## Related

- [[research/api-shape-drift-2026-07-10]] — the evidence this iteration acts on
- [[iterations/iteration-33-fixture-refresh]] — original recording plumbing
- [[iterations/iteration-48-record-pii-redaction]] — redaction layer
- [[dogfooding/dogfooding-playbook]]
- [[decision-log]]
