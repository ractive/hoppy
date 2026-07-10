---
title: Iter-67 — credential rotation & storage safety
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - security
  - storage
  - stream
  - pull-zone
status: in-progress
branch: iter-67/credential-rotation
priority: 1
depends-on: iter-66/spec-refresh-drift-fixes
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/storage
  - research/api-coverage-2026-07/stream
  - research/api-coverage-2026-07/pullzone-misc
---

# Iter-67 — credential rotation & storage safety

## Why

Per [[research/api-coverage-gap-analysis-2026-07]] §4, all six credential
rotation endpoints are missing: a leaked storage password, Stream library
key, or pull-zone security key cannot be rotated from the CLI at all.
Small, high-value, security-relevant. Bundles the remaining storage
safety flags from [[research/api-coverage-2026-07/storage]].

**Carried over from [[iterations/iteration-66-spec-refresh-drift-fixes]]:**
all 5 rotation/delete endpoint paths and params referenced below
(`resetPassword`, `resetReadOnlyPassword`, `resetApiKey`,
`resetReadOnlyApiKey`, `resetSecurityKey`, `deleteLinkedPullZones`) were
spot-checked against the iter-66-refreshed `specs/core-platform.json` and
match this plan's paths/params exactly — no further drift expected there.
This run has no live API access (unattended), so any AC that would
normally be "verify live" should be satisfied by spec/unit/e2e evidence
and, where a genuine live check is needed, deferred to a tracked backlog
item instead (see iter-66's `db-fork-group-field-drift` for the pattern).

## Scope

### 1. Storage zone password rotation

- [x] `storage-zone reset-password` → `POST /storagezone/{id}/resetPassword`
- [x] `storage-zone reset-read-only-password` →
  `POST /storagezone/resetReadOnlyPassword?id=` (note: id is a query param)
- [x] Confirmation prompt + `-y`; new secrets redacted unless `--reveal`

### 2. Video library key rotation

- [x] `stream library reset-api-key` → `POST /videolibrary/{id}/resetApiKey`
- [x] `stream library reset-read-only-api-key` →
  `POST /videolibrary/{id}/resetReadOnlyApiKey`
- [x] Same confirm/redact pattern as storage-zone

### 3. Pull-zone security key rotation

- [x] `pull-zone reset-security-key` →
  `POST /pullzone/{id}/resetSecurityKey` (completes the half-covered
  `--zone-security-enabled` story)

### 4. Storage upload integrity

- [x] `storage upload --checksum` — send the SHA-256 `Checksum` header;
  client param exists (`upload_file(..., checksum)`), CLI passes `None`
  (`commands/storage.rs:101`). Support `--checksum <hex>` and consider
  computing it locally when the flag is given without a value.
  **Spec note (`specs/storage.json`): the header value must be uppercase
  hex** — the client doc comment already says so; if computing locally,
  uppercase the digest before sending. If hashing the upload body
  locally, keep it streaming (hash while reading the chunked body, not
  by buffering the whole file) per the CLAUDE.md streaming rule reinforced
  in iter-66's `download_file_streaming`

### 5. Storage zone delete safety

- [x] **Spec check (confirmed against the iter-66-refreshed
  `specs/core-platform.json`): `deleteLinkedPullZones` defaults to
  `true` upstream.** `delete_storage_zone` (`core/client.rs:440`) sends
  no query param today, so bunny.net already deletes linked pull zones
  by default on every `storage-zone delete` — silently. Add the
  `deleteLinkedPullZones` query param to `delete_storage_zone` and expose
  a `--keep-linked-pull-zones` (or `--delete-linked-pull-zones <bool>`)
  flag that lets the caller opt OUT of the destructive default, not just
  opt in; the confirmation prompt should say explicitly whether linked
  pull zones will be deleted

### 6. `storage rm` directory-delete semantics

- [x] Preserve the trailing slash so `storage rm --remote-path images/`
  targets the directory URL form (recursive delete); today
  `split_remote_path` trims it and `file_url` never re-adds it
- [x] Distinct confirmation wording for recursive directory deletes

## Out of scope

- Access-key selection (using `ReadOnlyPassword` for read-only ops) —
  file as backlog if friction shows up during dogfooding

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated for all new commands/flags
- [x] `hyalo lint` clean on touched knowledgebase files
