---
title: Iter-74 — pull-zone body completeness
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - pull-zone
status: in-progress
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

**Lesson carried from [[iteration-73-video-library-settings]]**: that
iteration's field-inventory discipline (diffing the full spec body against
the CLI arg surface, not just what the handler consumes) again caught
every gap before review — apply the same rigor per group below (§1-§4),
confirming each prop is both a struct field *and* a reachable flag before
ticking the box. Also: `pull-zone update` was already the largest command
variant in the CLI before this iteration; iter-73 needed
`#[allow(clippy::large_enum_variant)]` on its own biggest `Update` variant
once flag count grew similarly, so expect (and pre-emptively allow, with a
one-line justification comment) the same clippy finding here rather than
being surprised by it at review time.

## Scope

All body props below need `UpdatePullZone` struct fields
(`core/types.rs`) + flags on `pull-zone update` + serialize tests.

### 1. Error-page group (5 props)

- [x] `ErrorPageEnableCustomCode`, `ErrorPageCustomCode`,
  `ErrorPageEnableStatuspageWidget`, `ErrorPageStatuspageCode`,
  `ErrorPageWhitelabel`

### 2. Preloading-screen group (7 props)

- [x] `PreloadingScreenEnabled`, `PreloadingScreenCode`,
  `PreloadingScreenLogoUrl`, `PreloadingScreenShowOnFirstVisit`,
  `PreloadingScreenTheme`, `PreloadingScreenCodeEnabled`,
  `PreloadingScreenDelay`

### 3. Edge/middleware scripting wiring

- [x] `EdgeScriptId`, `MiddlewareScriptId` (+ `EdgeScriptExecutionPhase`)
  — attach `hoppy script` output to a pull zone

### 4. Magic Containers origin

- [x] `MagicContainersAppId`, `MagicContainersEndpointId` — point a zone
  at a `hoppy container` app/endpoint

### 5. Private key type endpoint

- [x] `pull-zone hostname update-key-type` →
  `POST /pullzone/{id}/updatePrivateKeyType` (RSA/EC switch; no client
  method today)

### 6. External-DNS certificate flow

- [x] `POST /pullzone/requestExternalDnsCertificate` +
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

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated; `fixtures/core/pullzone_get.json` carries new keys
- [x] `hyalo lint` clean on touched knowledgebase files
