---
title: Iter-74 — pull-zone body completeness
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - pull-zone
status: planned
branch: iter-74/pull-zone-body-completeness
priority: 3
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/pullzone-misc
---

# Iter-74 — pull-zone body completeness

## Why

Per [[research/api-coverage-2026-07/pullzone-misc]], `pull-zone update`
covers 122/143 body props; the 21 unreachable ones cluster into whole
feature areas — error pages, preloading screen, and the script/container
wiring that blocks connecting `hoppy script` / `hoppy container` output
to a zone. Plus two missing endpoints and the new external-DNS cert flow.

## Scope

All body props below need `UpdatePullZone` struct fields
(`core/types.rs`) + flags on `pull-zone update` + serialize tests.

### 1. Error-page group (5 props)

- [ ] `ErrorPageEnableCustomCode`, `ErrorPageCustomCode`,
  `ErrorPageEnableStatuspageWidget`, `ErrorPageStatuspageCode`,
  `ErrorPageWhitelabel`

### 2. Preloading-screen group (7 props)

- [ ] `PreloadingScreenEnabled`, `PreloadingScreenCode`,
  `PreloadingScreenLogoUrl`, `PreloadingScreenShowOnFirstVisit`,
  `PreloadingScreenTheme`, `PreloadingScreenCodeEnabled`,
  `PreloadingScreenDelay`

### 3. Edge/middleware scripting wiring

- [ ] `EdgeScriptId`, `MiddlewareScriptId` (+ `EdgeScriptExecutionPhase`)
  — attach `hoppy script` output to a pull zone

### 4. Magic Containers origin

- [ ] `MagicContainersAppId`, `MagicContainersEndpointId` — point a zone
  at a `hoppy container` app/endpoint

### 5. Private key type endpoint

- [ ] `pull-zone hostname update-key-type` →
  `POST /pullzone/{id}/updatePrivateKeyType` (RSA/EC switch; no client
  method today)

### 6. External-DNS certificate flow

- [ ] `POST /pullzone/requestExternalDnsCertificate` +
  `POST /pullzone/completeExternalDnsCertificate` (new July-spec ops) as
  a two-step `pull-zone hostname` cert flow with clear help text on
  ordering

## Out of scope

- `LogFormat` / `LogForwardingFormat`, `BunnyAiImageBlueprints`,
  `OriginType` — remaining low-value props, backlog
- Edge-rule trigger parser gaps (`PatternMatchingType`, `Parameter1`,
  `ExtraActions`, `OrderIndex`) — separate backlog item
- `checkavailability` + `pull-zone count` — done in
  [[iteration-71-dns-completeness]]

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated; `fixtures/core/pullzone_get.json` carries new keys
- [ ] `hyalo lint` clean on touched knowledgebase files
