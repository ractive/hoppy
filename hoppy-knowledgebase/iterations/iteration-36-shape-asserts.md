---
title: Iter-36 — shape-first wiremock asserts (unblock fixture refresh)
type: iteration
date: 2026-05-14
tags:
  - iteration
  - testing
  - fixtures
status: completed
branch: iter-36/shape-asserts
---

# Iter-36 — shape-first wiremock asserts

## Why

Iter-34 shipped the fixture-refresh tool, but the 2026-05-14 dogfooding
round failed at the last step: applying 14 drifted fixtures broke 7
offline tests because they assert on **hand-authored fixture values**
(`balance == 42.50`, `id == 1001`, specific bandwidth counts) rather
than on the response *shape*. The refresh tool can't be used as a real
refresh until these tests are loosened.

See [[backlog/fixture-tests-assert-on-hardcoded-values]] for the
diagnosis.

## Target shape

After this iteration:
- The 7 failing wiremock tests assert on response *shape* (types,
  invariants, presence) — not on hand-authored values.
- A future fixture refresh (`fixture-refresh --apply` after a live
  sweep) can land drift without breaking the offline suite.
- The pattern is documented so new wiremock tests adopt it.

## Scope

### 1. Identify all value-coupled asserts in the 4 affected files

The 7 failing tests live in:
- `crates/bunny-net-api/tests/core/e2e/billing_api.rs`
- `crates/bunny-net-api/tests/core/e2e/pullzone_api.rs`
- `crates/bunny-net-api/tests/core/e2e/dns_api.rs`
- `crates/bunny-net-api/tests/core/e2e/statistics_api.rs`

- [x] For each test that asserts on response values, classify each
      assertion as either:
  - **Value-coupled** (must change): asserts on a specific number/string
    that came from the hand-authored fixture and could plausibly drift.
  - **Shape-coupled** (keep): asserts that test serde behaviour itself
    — defaults, optional fields, redacted fields, partial-response
    fallbacks.
  - **Wire-format** (keep): asserts on the *request* body or query
    string — those test what the client sent, not what the server
    returned, and don't depend on the fixture.

### 2. Rewrite value-coupled asserts as shape-first

Replace `assert_eq!(billing.balance, 42.50)` with one of:

- `assert!(billing.balance >= 0.0);` — invariant.
- `assert!(billing.balance.is_finite());` — type/serde correctness.
- `assert!(billing.this_month_charges >= 0.0);` — same.

For IDs: drop the `== 1001` assertion; keep `assert_eq!(zone.id, fixture_id)`
only when the test sets up a specific ID expectation (e.g. round-tripping
the request URL). For paginated responses: keep `assert!(!items.is_empty())`
instead of `assert_eq!(items.len(), 2)` when the count came from the
fixture.

- [x] Rewrite billing tests (3 fields with hand-authored values).
- [x] Rewrite pullzone tests (ID match against fixture).
- [x] Rewrite dns_api tests (paginated counts + zone IDs).
- [x] Rewrite statistics tests (bandwidth values).

### 3. Cover obvious neighbours

The 7 are the ones that *happened* to break this round. Other tests in
the same 4 files probably assert on fixture values that haven't drifted
*yet* — they'll break the next time a different field changes upstream.

- [x] Skim each of the 4 files; rewrite obviously value-coupled asserts
      even if they didn't fail this round.
- [x] Don't touch tests that are checking serde defaults or partial-response
      fallbacks — those legitimately need specific values.

### 4. Document the pattern

- [x] Add a short section to `dogfooding/dogfooding-playbook.md` under
      "Refreshing fixtures" explaining shape-first asserts and why
      they matter.
- [x] OR add it to a new `hoppy-knowledgebase/decision-log.md` entry —
      whichever fits better.

### 5. Verify

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.

### 6. Re-run fixture refresh (optional, end of iter)

- [ ] If time permits, run a live sweep + `fixture-refresh --apply` to
      prove the offline suite now passes against refreshed fixtures.
      Commit the resulting drift.

## Out of scope

- Rewriting wiremock tests in files outside the 4 affected ones — file
  follow-up backlog items if other files break a future refresh.
- Switching to a different testing framework.
- Changing the recording framework.

## Acceptance

- All 4 affected test files use shape-first asserts on fixture-derived
  values.
- `cargo test --workspace --quiet` passes against current fixtures
  (baseline) AND against a hypothetical refreshed set (verified by
  applying iter-34's last drift report and re-running tests).
- Pattern documented somewhere durable.

## Related

- [[backlog/fixture-tests-assert-on-hardcoded-values]] — the
  motivating backlog item.
- [[iteration-34-fixture-mapper]] — the tool that surfaced this.
