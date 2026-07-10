---
title: Iter-73 — video library settings & stream odds-and-ends
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - stream
  - video-library
status: in-progress
branch: iter-73/video-library-settings
priority: 3
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/stream
---

# Iter-73 — video library settings

## Why

Per [[research/api-coverage-2026-07/stream]], Video Library management is
the weakest domain (5/20 covered): `stream library update` reaches only
4 body fields while the live API accepts dozens, and referrers,
watermarks, and languages are unreachable end-to-end.

**Lesson carried from [[iteration-72-shield-new-surface]]**: diffing the
full spec field/param list against the CLI arg surface (not just what the
handler consumes) again caught every gap before review — apply the same
discipline to §1's "field inventory from the live API/dashboard" task,
since the spec body is `<unknown>` there and it's easy to under-cover.
Also: §3's watermark upload is an actual binary image blob (unlike
iter-72's small custom-page HTML, which was fine to buffer as a
`String`) — per `CLAUDE.md`'s streaming-body rule, wire it through
`reqwest::Body::wrap_stream` / a file stream rather than reading the
whole image into memory first.

## Scope

### 1. Full library update surface

- [x] Extend `UpdateVideoLibrary` + `stream library update` flags beyond
  Name/AllowDirectPlay/EnableMP4Fallback/HasWatermark: enabled
  resolutions, output codecs, player config (key color, captions font
  size, …), `WebhookUrl`, `KeepOriginalFiles`, `AllowEarlyPlay`,
  `EnableDRM`, transcription defaults (`POST /videolibrary/{id}`)
- [x] Field inventory from the live API/dashboard where the spec body is
  `<unknown>`; document verified fields in the KB

### 2. Referrer allow/block ops (4)

- [x] `stream library referrer allow` / `block` / `remove-allowed` /
  `remove-blocked` → `POST /videolibrary/{id}/addAllowedReferrer`,
  `addBlockedReferrer`, `removeAllowedReferrer`, `removeBlockedReferrer`
  (mirror the `pull-zone referrer` command shape)

### 3. Watermark

- [x] `stream library watermark set` → `PUT /videolibrary/{id}/watermark`
  (image upload; stream the body)
- [x] `stream library watermark delete` →
  `DELETE /videolibrary/{id}/watermark`

### 4. Languages

- [x] `stream library languages` → `GET /videolibrary/languages`

### 5. Player-facing stream endpoints

- [x] `GET /OEmbed`, `GET /library/{lib}/videos/{vid}/play`, and
  `GET .../play/heatmap` → new `stream video` subcommands (oembed,
  play-data, play-heatmap or similar)

### 6. Video metadata update

- [x] Add `chapters`, `moments`, `metaTags` to `UpdateVideo` +
  `stream video update` (nested arrays — JSON file input à la
  `--config-json` is acceptable)

## Out of scope

- Live-stream thumbnail/watermark ops (`PUT/DELETE
  /videolibrary/{id}/live/*`, 4 ops) — verify liveness first, backlog
- `resetApiKey` / `resetReadOnlyApiKey` — done in
  [[iteration-67-credential-rotation]]
- TUS resumable upload — [[iteration-77-stream-tus-upload]]

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated for all new commands/flags
- [x] `hyalo lint` clean on touched knowledgebase files
