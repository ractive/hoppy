---
title: "recording filenames don't map to hand-authored wiremock fixtures"
type: backlog
date: 2026-05-14
status: planned
priority: high
origin: dogfooding-2026-05-14 (iter-33 §5 dogfooding round)
---

# `HOPPY_RECORD_DIR` writes a parallel filename scheme

Iter-33 shipped per-domain fixture routing and idempotent overwrites, but
the live-api sweep on 2026-05-14 revealed the refresh promise is unmet:

- Existing 168 hand-authored fixtures use descriptive snake_case names:
  `core/billing_get.json`, `core/dnszone_create.json`, …
- The recording framework writes auto-derived `<METHOD>_<path-segments>.json`:
  `core/GET_billing.json`, `core/POST_dnszone.json`, …
- Result: a full live sweep wrote **205 new files** and overwrote **0**
  existing ones. The refresh mechanism is effectively additive, not
  refreshing.

## Why this needs a mapper, not a rename

A blanket "switch recording to descriptive names" doesn't work — there is
no 1:1 between (method, path) and the descriptive name a test author chose.
Multiple existing fixtures often share a path (paginated vs first-page,
success vs 404 variants), and ID-bearing paths bake the live test's IDs
into the filename so they can't match a static fixture name verbatim.

The bridge is the **test code itself**: every `include_str!("…/core/foo.json")`
is paired with a `mock(method(...)).path(...)` call that defines exactly
which HTTP request that fixture serves.

## Proposed fix (sketch)

A small helper (Rust binary in `crates/hoppy-cli/src/bin/` or a shell
script in `hoppy-knowledgebase/dogfooding/`):

1. Grep the wiremock test sources for `include_str!("…/<domain>/<name>.json")`
   and the surrounding `method(...)` / `path(...)` calls.
2. Build a table `descriptive_name → (METHOD, /path/template)`.
3. Run the live sweep into a temp dir.
4. For each existing fixture, look up its (METHOD, path), template-replace
   any test-generated IDs back to placeholders, find the matching recording.
5. Diff bytes. If different, overwrite the descriptive-name fixture with
   the fresh body. Print a summary.

## Out of scope

- Auto-redaction of account-specific values.
- Recording non-2xx error fixtures.
- Replacing the recording framework's filename scheme; it's reasonable for
  ad-hoc `--record <dir>` debugging.

## Related

- [[../iterations/iteration-33-fixture-refresh]] §5 (this is the §5
  blocker that prevented a true refresh round)
