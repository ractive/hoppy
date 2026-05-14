---
title: Iter-34 — fixture mapper for live-api refresh
type: iteration
date: 2026-05-14
tags:
  - iteration
  - testing
  - fixtures
  - tooling
status: in-progress
branch: iter-34/fixture-mapper
---

# Iter-34 — fixture mapper

## Why

Iter-33 shipped the plumbing for `HOPPY_RECORD_DIR`-driven fixture
recording, but the 2026-05-14 dogfooding round
([[iteration-33-fixture-refresh]] §5) revealed that the recording
filenames (`GET_dnszone_50001.json`) don't match the hand-authored
wiremock fixtures (`dnszone_get.json`) — a full live sweep wrote 205
new files and overwrote zero existing ones. The "refresh" mechanism is
additive, not refreshing.

See [[../backlog/fixture-recording-name-mismatch]] for the full diagnosis.

## Target shape

```sh
HOPPY_RECORD_DIR="$(pwd)/fixtures-recorded" BUNNY_API_KEY=<test> \
    cargo test --workspace --features live-api -- --test-threads=1
cargo run --bin fixture-refresh -- --recorded fixtures-recorded --apply
# → diff existing fixtures/ against recorded responses by (method, path)
# → overwrite only the descriptive-name fixtures that drifted
# → print one line per overwrite, one per unmappable recording
```

A second sweep should produce **no diff** (idempotent), and every change
git surfaces should be a real API drift.

## Scope

### 1. Static analysis of wiremock fixture references

- [x] Scan all `crates/**/tests/**/*.rs` for `include_str!("…/<domain>/<name>.json")`
      and walk the surrounding scope to find the paired `method(...)` and
      `path(...)` / `path_regex(...)` calls.
- [x] Produce a JSON-serialisable table:
      `descriptive_name → { method, path_template }` keyed by domain.
- [x] Handle the common shapes: `Mock::given(method("GET")).and(path("/billing"))`,
      `path_regex(r"^/dnszone/\d+$")`, etc.
- [x] Skip fixtures referenced from non-test code (the framework's own
      sample data, doc tests).

### 2. Recording → existing fixture matcher

- [x] Given the table from §1 and a directory of recorded
      `<METHOD>_<path-segments>.json` files, match recordings to
      descriptive names by templating the path back from segments
      (numeric segments → `\d+`, UUID-shaped → `[uuid]`).
- [x] When multiple existing fixtures share a (method, path) — paginated
      vs first-page, success vs 404 — refuse to overwrite and report the
      collision so a human can resolve it.

### 3. Diff and apply

- [x] For each mapped (descriptive_name → recording) pair, byte-compare.
      Identical → skip silently. Different → record as "drift".
- [x] `--dry-run` mode (default) prints a summary table:
      `<domain>/<name>.json — drift (N bytes)` or `unmappable: <recording>`.
- [x] `--apply` overwrites the descriptive-name fixtures. Untouched on
      collisions / unmappables.

### 4. Surface as a tool

- [x] New binary in `crates/hoppy-cli/src/bin/fixture_refresh.rs` (so it
      lives inside the existing crate, not a separate package).
- [x] CI does NOT run this; it's a manual dogfooding tool.
- [x] Document in `dogfooding/dogfooding-playbook.md` "Refreshing fixtures"
      section: replace the bare `HOPPY_RECORD_DIR=fixtures/` recipe with
      "record into a scratch dir, then run `fixture-refresh --apply`".

### 5. Dogfooding round

- [ ] Build release, set `BUNNY_API_KEY="$TEST_BUNNY_API_KEY"`.
- [ ] Run the live sweep into a scratch directory (not `fixtures/`).
- [ ] `cargo run --bin fixture-refresh -- --recorded <scratch>` → review
      dry-run output for surprises (collisions, unmappables).
- [ ] `--apply`, then `cargo test --workspace --quiet` to prove the
      offline suite still passes against the refreshed fixtures.
- [ ] Commit the resulting drift in the same PR.

## Out of scope

- Auto-redaction of account-specific values. The dogfooding playbook's
  manual redaction step still applies.
- Recording non-2xx error fixtures.
- Renaming descriptive fixtures to match the recording scheme — too much
  test-code churn for too little benefit.
- Auto-applying drift on CI. This stays a manual tool.

## Risks and mitigations

- **Path-template inference is fragile.** Mitigation: §2 collisions
  refuse to overwrite, surface for human review. Worst case is "tool
  reports lots of unmappables and applies nothing" — same as today.
- **`path_regex` is harder to invert than `path`.** Mitigation: support
  the small set of regex shapes our tests actually use; bail out on
  anything else.
- **API drift floods the diff.** Mitigation: review per-domain. If any
  one domain has unexpectedly large drift, file a separate backlog
  item before committing the refresh.

## Acceptance

- `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
  cargo test --workspace --quiet` clean.
- Running `fixture-refresh --recorded <dir>` twice in a row produces no
  diff on the second run.
- At least one dogfooded refresh round completed; resulting drift
  reviewed and committed in the same PR.
- Playbook updated with the new flow.

## Related

- [[../backlog/fixture-recording-name-mismatch]] — the diagnosis driving
  this iteration.
- [[iteration-33-fixture-refresh]] — shipped the recording plumbing this
  iteration consumes.
- [[../dogfooding/dogfooding-playbook]] — the safe loop the dogfooding
  round runs inside.
