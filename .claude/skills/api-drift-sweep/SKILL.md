---
name: api-drift-sweep
description: >
  Run a full live-API drift sweep: record all live-API test responses into a
  scratch directory, generate a read-only shape/leak report against the
  checked-in fixtures, file the findings in the knowledgebase, and clean up.
  Use this skill when the user says /api-drift-sweep, "run a drift sweep",
  "check for API drift", "refresh the fixture report", "run the live API
  sweep", or wants to know whether the bunny.net API grew fields the client
  doesn't model yet. This is the successor to the retired fixture-refresh
  --apply workflow: it NEVER modifies fixtures/.
---

# API drift sweep

Runs the live-API recording sweep and turns it into a knowledgebase drift
report. The sweep is **read-only with respect to `fixtures/`** — fixture
updates only ever happen inside a deliberate iteration that changes client
types, fixtures, and tests together (see [[decision-log]]).

## Hard rules

- **Use `TEST_BUNNY_API_KEY` and nothing else.** Every live command below
  gets `BUNNY_API_KEY="$TEST_BUNNY_API_KEY"`. If the variable is unset or
  `hoppy auth check` fails, STOP and ask the user. Never fall back to any
  other key.
- **Never write to `fixtures/`.** The tooling has no code path for it;
  don't add one ad hoc.
- **Never commit the scratch recordings.** They are destroyed at the end.
- Destructive account cleanup (deleting leaked test resources) requires
  explicit user approval in-session; list first, delete only after a yes.

## Procedure

### 1. Pre-flight

```sh
cargo build --release
BUNNY_API_KEY="$TEST_BUNNY_API_KEY" ./target/release/hoppy auth check
```

Then a read-only leak scan of the account (pull zones, storage zones,
container apps, stream libraries, dns zones, scripts, dbs — `<noun> list`).
Report any leftovers. Known caveats: the container update-lifecycle test
leaks one `hpmc-*-upd` app per run, and orphaned shield zones cannot be
deleted (no API endpoint) — see
[[backlog/leaked-test-resources-cleanup-script]].

### 2. Recording sweep (background, ~10 min)

```sh
SCRATCH="$(pwd)/fixtures-recorded"
HOPPY_RECORD_DIR="$SCRATCH" BUNNY_API_KEY="$TEST_BUNNY_API_KEY" \
    cargo test --workspace --features live-api --quiet -- --test-threads=1
```

`--test-threads=1` is required (fixture filename races). One or two live
tests may flake (known: stream statistics 401 —
[[backlog/live-stream-statistics-401-retry]]); re-run just the failed test
filters once with the same env so their endpoints get recorded. Expect
shield endpoints gated by plan tier to record error envelopes on the test
account — that is normal and excluded from drift conclusions.

### 3. Shape + leak report

```sh
cargo run --release --bin fixture-refresh -- \
    --recorded fixtures-recorded --shape-report --out drift-report.md
```

Exit codes: `0` clean, `1` drift found, `2` leak-audit hits.

- **Exit 2 (leaks)**: stop and fix the redaction rules in
  `crates/bunny-net-api/src/recording/redact.rs` first (that is a bug),
  re-record, re-report. Never file a report containing leaked values.
- **Exit 1 (drift)**: continue below.

### 4. Verify the tree is untouched

`git status -- fixtures/` must be clean and
`cargo test --workspace --quiet` must be green. If either fails, something
wrote where it shouldn't — stop and investigate.

### 5. File the findings

Create `hoppy-knowledgebase/research/api-shape-drift-<YYYY-MM-DD>.md`
(frontmatter: `type: research`, `status: active`, dated) from the report:
per-surface summary of added/removed fields, the unmapped-endpoint list,
plan-tier caveats. Lint with `hyalo lint --file <path>`. Move
`drift-report.md` content into the note; don't leave stray report files in
the repo root.

### 6. Cleanup

- `rm -rf fixtures-recorded` (always — recordings may hold account IDs).
- Re-run the account leak scan; list anything the sweep leaked and ask the
  user before deleting (`container app delete --id <id> --cascade --yes`
  for apps).

### 7. Hand off

End with a ranked list of iteration candidates derived from the drift
report (biggest unmodeled surfaces first), linking the research note. Do
NOT update fixtures, types, or tests here — that's the next iteration's
job.

## Related

- [[dogfooding/dogfooding-playbook]] — rationale, redaction rules, caveats
- [[iterations/iteration-78-api-drift-radar]] — where this workflow landed
- [[research/api-shape-drift-2026-07-10]] — first (hand-made) drift report
