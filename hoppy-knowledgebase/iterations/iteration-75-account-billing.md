---
title: Iter-75 — account & billing surface
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - billing
  - account
status: planned
branch: iter-75/account-billing
priority: 4
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/pullzone-misc
---

# Iter-75 — account & billing

## Why

Per [[research/api-coverage-2026-07/pullzone-misc]], 11 of 14 core misc
ops are missing — an entire account/admin command group: API keys,
billing summary/invoices, reference data, global search, and the audit
log. Only `GET /billing` is reachable today (as `auth check`).

## Scope

### 1. API keys

- [ ] `hoppy apikey list` → `GET /apikey` — key values redacted unless
  `--reveal`

### 2. Billing

- [ ] `hoppy billing summary` → `GET /billing/summary`
- [ ] `hoppy billing payment-requests` → `GET /billing/payment-requests`
- [ ] `hoppy billing invoice-pdf --record-id --output` →
  `GET /billing/summary/{billingRecordId}/pdf` (binary; stream to file)
- [ ] `hoppy billing payment-request-pdf --id --output` →
  `GET /billing/payment-request-invoice/{id}/pdf` (binary; stream to
  file)

### 3. Reference data

- [ ] `hoppy region list` → `GET /region` (core `/region`, distinct from
  the containers `/regions`)
- [ ] `hoppy country list` → `GET /country` — documents valid values for
  `pull-zone update --blocked-countries`

### 4. Global search

- [ ] `hoppy search <query>` → `GET /search` (cross-resource search;
  map its pagination params)

### 5. User audit log

- [ ] `hoppy user audit --date …` → `GET /user/audit/{date}` with all 7
  query params from the spec

## Out of scope

- `POST /user/closeaccount` — destructive, low CLI value; deliberately
  unexposed (gap-analysis decision)
- `GET /billing/affiliate` — niche; backlog if requested

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new command (`tests/e2e/` pattern)
- [ ] PDF downloads streamed, never buffered whole (project rule)
- [ ] Help text present for all new command groups
- [ ] `hyalo lint` clean on touched knowledgebase files
