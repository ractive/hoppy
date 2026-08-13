---
title: Iter-84 — backlog burn-down (reveal threading, db slug limit, db region vocab)
type: iteration
date: 2026-08-13
tags:
  - iteration
  - dx
  - backlog
  - database
status: in-progress
branch: iter-84/backlog-fixes
---

# Iter-84 — backlog burn-down: reveal threading, db slug limit, db region vocab

## Why

The three remaining open backlog items (all filed during iter-81 dogfooding,
all low priority individually, but together they close the backlog):

- [[backlog/core-client-reveal-not-threaded-everywhere]]
- [[backlog/db-create-slug-length-mismatch]]
- [[backlog/db-group-create-region-vocab]]

## Design

### 1. `--reveal` threading (core-client commands)

The backlog item predates iter-82's `ClientOpts` refactor. Today
`auth::core_client(opts)` and `auth::core_client_with_reveal(opts)` are
**identical** (the former delegates to the latter; both read
`opts.reveal_secrets`). The actual bug is at call sites: the handlers for
auth-whoami, billing/region/country/search/user (account.rs), dns, purge,
statistics, and the nested core client in container.rs build `ClientOpts`
with `..Default::default()`, silently pinning `reveal_secrets: false` — so
`--debug --reveal` still prints `<set, length=N>` placeholders there.
Fails safe, but under-delivers.

Fix, compiler-enforced as the backlog item asks:

- Remove the `Default` derive from `ClientOpts` so every construction must
  state `reveal_secrets` (and every other field) explicitly — no silent
  pinning possible for future call sites.
- Delete the redundant `core_client_with_reveal` alias; keep a single
  `core_client(opts)` (mechanical rename at ~6 call sites). Same for other
  `*_client_with_reveal` aliases only if they are equally redundant —
  otherwise leave them.
- Thread the global `cli.reveal` flag from `main.rs` into the handlers
  that don't receive it yet and pass it as `reveal_secrets`.

### 2. `db create` slug-length validation vs real upstream limit

Local validator (`database.rs`, `SLUG_MAX_LEN = 24`) claims to prevent the
upstream 500 but doesn't: 19 chars passed validation and got HTTP 500;
13 chars worked (iter-81 field report).

- **Live probe first** (test account, dogfooding playbook): binary-search
  the boundary in 14–19 with throwaway `hoppy-test-…` slugs in the default
  group, deleting each db immediately (cleanup script style). Note: with
  only one group available the "limit is on `len(group_ulid)+1+len(slug)`
  hostname" hypothesis can't be isolated from a plain slug-length limit —
  record the measured effective limit and state the caveat in the backlog
  item's resolution note.
- Encode the measured limit in `SLUG_MAX_LEN`, update the comment, error
  message, and the `--slug` help text in `cli.rs`. `db fork`'s target slug
  goes through the same `validate_slug` and inherits the fix.

### 3. `db group create` region vocabulary validation

`--primary-region` takes short uppercase codes (`DE`, `AMS`, …);
`--storage-region` takes AWS-style ids (`eu-west-1`, …). Neither is
documented; wrong casing produces a raw JSON-schema dump or an upstream
500. Both vocabularies are served by `GET` config
(`DatabaseClient::get_config()` → `regions_available` /
`storage_region_available`).

- Pre-flight validation in the `db group create` handler: call
  `get_config()` (read-only — still executes under `--dry-run`, keeping
  the preview truthful, same pattern as storage-zone password resolution),
  and check the provided values against the live vocabularies
  case-sensitively. On mismatch, fail before the mutating call with the
  valid values listed (and a did-you-mean when only the casing is wrong).
  No hardcoded region lists — the vocabularies move with the API.
- Help text for both flags names the vocabulary shape and points at
  `hoppy db config show` for the full list.

## Tasks

- [x] Branch `iter-84/backlog-fixes`; set status `in-progress`
- [x] Reveal: drop `Default` from `ClientOpts`, collapse
      `core_client_with_reveal` into `core_client`, thread `cli.reveal`
      into auth/account(billing,region,country,search,user)/dns/purge/
      statistics/container-core handlers
- [x] Live probe: binary-search real slug limit on test account
      (`hoppy-test-…` prefix, immediate cleanup, TEST_BUNNY_API_KEY)
- [x] Encode measured `SLUG_MAX_LEN` + fix comment, error, `--slug` help
- [x] `db group create`: `get_config()` pre-flight validation for
      `--primary-region` / `--storage-region` + help-text vocabularies
- [x] Tests: reveal threading (e2e: `--debug --reveal` shows raw body on a
      previously-pinned command), slug validator boundary cases, region
      pre-flight (valid, invalid, casing-only mismatch — wiremock)
- [x] CHANGELOG entries; command-tree notes if flag help changed
- [x] Quality gates: `cargo fmt` → `cargo clippy --workspace --all-targets
      -- -D warnings` → `cargo test --workspace --quiet`
- [x] Dogfood: `--debug --reveal` on a core command, `db create` with an
      over-limit slug (local rejection), `db group create` with bad casing
- [x] Mark the three backlog items `status=resolved` with resolution notes
- [x] PR `iter-84/backlog-fixes`, self-review diff first

## Acceptance criteria

- [x] `--debug --reveal` prints unredacted bodies on billing/dns/purge/
      statistics/region/country/search/user/auth-whoami commands
- [x] `ClientOpts` cannot be constructed without an explicit
      `reveal_secrets` (no `Default`)
- [x] `db create` locally rejects slugs the API would 500 on, and accepts
      lengths the API accepts (boundary verified live)
- [x] `db group create` with an unknown or wrongly-cased region fails
      before the mutating request, listing valid values
- [x] All three backlog items resolved; quality gates pass

## Out of scope

- Redaction behavior itself (what counts as a secret) — only the flag
  threading
- Hardcoding region vocabularies locally (validation is live via
  `get_config()`)

## References

- [[iterations/iteration-81-backlog-burndown]] — origin of all three items
- [[dogfooding/dogfooding-playbook]] — live-probe discipline
