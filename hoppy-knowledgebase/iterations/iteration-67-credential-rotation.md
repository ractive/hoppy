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
status: planned
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

## Scope

### 1. Storage zone password rotation

- [ ] `storage-zone reset-password` → `POST /storagezone/{id}/resetPassword`
- [ ] `storage-zone reset-read-only-password` →
  `POST /storagezone/resetReadOnlyPassword?id=` (note: id is a query param)
- [ ] Confirmation prompt + `-y`; new secrets redacted unless `--reveal`

### 2. Video library key rotation

- [ ] `stream library reset-api-key` → `POST /videolibrary/{id}/resetApiKey`
- [ ] `stream library reset-read-only-api-key` →
  `POST /videolibrary/{id}/resetReadOnlyApiKey`
- [ ] Same confirm/redact pattern as storage-zone

### 3. Pull-zone security key rotation

- [ ] `pull-zone reset-security-key` →
  `POST /pullzone/{id}/resetSecurityKey` (completes the half-covered
  `--zone-security-enabled` story)

### 4. Storage upload integrity

- [ ] `storage upload --checksum` — send the SHA-256 `Checksum` header;
  client param exists (`upload_file(..., checksum)`), CLI passes `None`
  (`commands/storage.rs:101`). Support `--checksum <hex>` and consider
  computing it locally when the flag is given without a value

### 5. Storage zone delete safety

- [ ] `storage-zone delete --delete-linked-pull-zones` — add the
  `deleteLinkedPullZones` query param to `delete_storage_zone`
  (`core/client.rs:440`) and expose the flag

### 6. `storage rm` directory-delete semantics

- [ ] Preserve the trailing slash so `storage rm --remote-path images/`
  targets the directory URL form (recursive delete); today
  `split_remote_path` trims it and `file_url` never re-adds it
- [ ] Distinct confirmation wording for recursive directory deletes

## Out of scope

- Access-key selection (using `ReadOnlyPassword` for read-only ops) —
  file as backlog if friction shows up during dogfooding

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated for all new commands/flags
- [ ] `hyalo lint` clean on touched knowledgebase files
