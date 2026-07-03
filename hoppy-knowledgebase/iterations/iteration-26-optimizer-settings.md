---
title: Iteration 26 — Bunny Optimizer Pull Zone settings
type: iteration
date: 2026-05-09
tags:
  - iteration
  - cdn
  - optimizer
  - pull-zone
status: completed
branch: iter-26/optimizer-settings
---

# Iteration 26 — Bunny Optimizer Pull Zone settings

**Goal:** Expose Bunny Optimizer configuration through hoppy. The Optimizer is one of bunny.net's headline features (image resize/WebP/quality, CSS/JS minify, watermarking, static HTML / WordPress, manipulation engine) and its full configuration surface lives on the `PullZoneModel` — set via the standard `POST /pullzone/{id}` update endpoint. hoppy currently surfaces zero Optimizer fields on the type or the CLI, which is the largest concrete API gap as of iter-25.

## Context

Gap analysis on 2026-05-09 against the Pull Zone schema in:
- <https://core-api-public-docs.b-cdn.net/docs/v3/public.json>
- <https://docs.bunny.net/openapi/bunnynet-api-1.json>

found 24 `Optimizer*` fields on `PullZoneModel`, set through the existing pull zone update endpoint. There is no separate `/optimizer/*` write endpoint; the only standalone Optimizer endpoint is read-only statistics at `/pullzone/{id}/optimizer/statistics`, which hoppy already exposes via `pull-zone statistics --type optimizer`.

Today, in `crates/bunny-api-core/src/types.rs`:
- `PullZone` (response struct, ~`:368`) has none of the Optimizer fields.
- `UpdatePullZone` (request struct, ~`:564`) has none of the Optimizer fields.

The CLI `pull-zone update` (`src/cli.rs:270`) exposes none of them.

This iteration adds the full set of writable Optimizer fields to the types and the CLI update path, so operators can configure Optimizer end-to-end without falling back to raw `curl`.

## Scope

### Field model — extend `PullZone` (response) and `UpdatePullZone` (request)

Add the following fields to the response struct in `crates/bunny-api-core/src/types.rs` (camelCase wire form via `serde rename`, snake_case Rust). All `Option<T>` to stay forward-compat:

**Master switches**
- [x] `optimizer_enabled: Option<bool>`
- [x] `optimizer_automatic_optimization_enabled: Option<bool>`

**Image dimensions & quality**
- [x] `optimizer_desktop_max_width: Option<i32>`
- [x] `optimizer_mobile_max_width: Option<i32>`
- [x] `optimizer_image_quality: Option<i32>`
- [x] `optimizer_mobile_image_quality: Option<i32>`

**Format & upscale**
- [x] `optimizer_enable_web_p: Option<bool>`
- [x] `optimizer_enable_upscaling: Option<bool>`

**Minify**
- [x] `optimizer_minify_css: Option<bool>`
- [x] `optimizer_minify_java_script: Option<bool>`

**Manipulation engine**
- [x] `optimizer_enable_manipulation_engine: Option<bool>`
- [x] `optimizer_classes: Option<String>`
- [x] `optimizer_force_classes: Option<bool>`

**Watermark**
- [x] `optimizer_watermark_enabled: Option<bool>`
- [x] `optimizer_watermark_url: Option<String>`
- [x] `optimizer_watermark_position: Option<OptimizerWatermarkPosition>` — repr-enum (top-left=0, top-right=1, bottom-left=2, bottom-right=3, center=4); unknown discriminants deserialise to `None` via `deserialize_repr_option`
- [x] `optimizer_watermark_offset: Option<f64>`
- [x] `optimizer_watermark_min_image_size: Option<i32>`

**Static HTML / WordPress**
- [x] `optimizer_static_html_enabled: Option<bool>`
- [x] `optimizer_static_html_word_press_path: Option<String>`
- [x] `optimizer_static_html_word_press_bypass_cookie: Option<String>`

**Prerender / Tunnel**
- [x] `optimizer_prerender_html: Option<bool>`
- [x] `optimizer_tunnel_enabled: Option<bool>`

**Read-only (response only — do NOT add to `UpdatePullZone`)**
- [x] `optimizer_pricing: Option<f64>` — server-set tier indicator returned as a float (e.g. `9.5`) despite docs suggesting an integer

Mirror the writable subset (everything except `optimizer_pricing`) onto `UpdatePullZone` with `#[serde(skip_serializing_if = "Option::is_none")]` so unset flags don't overwrite server state.

### CLI — `pull-zone update` flags

Add flags to `src/cli.rs` for every writable field above, grouped under a single `--help` section:

- [x] `--optimizer-enabled <bool>` and `--optimizer-automatic-optimization <bool>`
- [x] `--optimizer-desktop-max-width <px>` / `--optimizer-mobile-max-width <px>`
- [x] `--optimizer-image-quality <0-100>` / `--optimizer-mobile-image-quality <0-100>`
- [x] `--optimizer-webp <bool>` / `--optimizer-upscaling <bool>`
- [x] `--optimizer-minify-css <bool>` / `--optimizer-minify-js <bool>` *(use `js` in the flag, but serialise to `OptimizerMinifyJavaScript` on the wire)*
- [x] `--optimizer-manipulation-engine <bool>` / `--optimizer-classes <json>` / `--optimizer-force-classes <bool>`
- [x] `--optimizer-watermark <bool>` plus `--optimizer-watermark-url`, `--optimizer-watermark-position {top-left,top-right,bottom-left,bottom-right,center}`, `--optimizer-watermark-offset <pct>`, `--optimizer-watermark-min-image-size <px>`
- [x] `--optimizer-static-html <bool>` plus `--optimizer-static-html-wp-path`, `--optimizer-static-html-wp-bypass-cookie`
- [x] `--optimizer-prerender-html <bool>` / `--optimizer-tunnel <bool>`
- [x] All flags use `Option<bool>` with `clap::ArgAction::Set` so they're tri-state (unset / true / false). Don't use `--no-foo` pairs — they double the surface.
- [x] Add an `after_help` example block on `pull-zone update` showing a typical Optimizer enablement: `hoppy pull-zone update --id <id> --optimizer-enabled true --optimizer-webp true --optimizer-minify-css true --optimizer-minify-js true --optimizer-image-quality 80`.

### Watermark position enum

- [x] Add `OptimizerWatermarkPosition` repr-enum to `bunny-api-core/src/types.rs`. Use the `deserialize_repr_option` helper introduced in iter-19 so unknown future values fall back to `None`.
- [x] CLI flag accepts the kebab-case names listed above; serialise to the integer.

### Tests (wiremock — no live API)

- [x] Round-trip a captured `pull-zone get` response that includes Optimizer fields without panic. Add a fixture `fixtures/core/pullzone_get_with_optimizer.json` (synthesise from the spec — set every Optimizer field to a non-default value).
- [x] `pull-zone update --id 1 --optimizer-enabled true --optimizer-image-quality 80 --optimizer-webp true` sends a request body containing `{"OptimizerEnabled":true,"OptimizerImageQuality":80,"OptimizerEnableWebP":true}` and **only** those keys (no other Optimizer fields, no other keys whose flags weren't set).
- [x] `pull-zone update --id 1 --optimizer-enabled false` sends `{"OptimizerEnabled":false}` (verify `false` is not skipped by `skip_serializing_if`).
- [x] `pull-zone update --id 1 --optimizer-watermark-position center` serialises to the correct integer.
- [x] `pull-zone get` JSON output includes every Optimizer field present in the fixture, unchanged.

### Help text

- [x] Group all `--optimizer-*` flags under a clap heading (`#[command(next_help_heading = "Optimizer")]` on a sub-struct, or a manual help section).
- [x] Add a one-line `long_help` for each non-obvious flag (e.g. `--optimizer-classes` — JSON map of class names to URL params, see <https://docs.bunny.net/docs/optimizer-classes>).
- [x] Cross-reference: `pull-zone statistics --type optimizer` for reading Optimizer usage after enabling it.

### Documentation

- [x] Add a short note to `hoppy-knowledgebase/api/bunny-api-quirks.md` (create if absent): the watermark position enum is repr-based and may grow; `optimizer_pricing` is server-set and ignored on writes.
- [x] Update `hoppy-knowledgebase/decision-log.md`: chose to expose the full Optimizer surface in one iteration rather than ship incrementally, because the fields are all on one struct and partial coverage would be more confusing than no coverage.

## Implementation Notes

- The 24 fields all live on a single struct, so this is a fan-out of mostly mechanical work plus one new repr-enum (watermark position). No new HTTP endpoints, no new auth paths.
- `OptimizerMinifyJavaScript` (note the long form on the wire) is a serialisation-naming gotcha — verify with a wiremock test that the request body uses `OptimizerMinifyJavaScript` exactly, not `OptimizerMinifyJs` or similar.
- Look at iter-19's enum forward-compat work (`deserialize_repr_option`) for the watermark-position pattern; reuse the helper, don't reinvent.
- `pull-zone create` is intentionally **out of scope**. Optimizer is typically configured after a Pull Zone exists, and adding 24 flags to `create` would bloat the command. If real-world users want create-time config, follow up in a later iteration.
- The bunny.net spec sometimes changes Optimizer field names between versions (e.g. older docs use `OptimizerEnableWebp` lowercase-p). Verify against a live `pull-zone get` response before committing the wire names — capture the response as a fixture and let the round-trip test be the source of truth.

## Suggested test cases

1. Enable optimizer end-to-end: `pull-zone update --id <id> --optimizer-enabled true --optimizer-webp true --optimizer-image-quality 75`. Assert wire body matches expected JSON.
2. Disable a single field: `pull-zone update --id <id> --optimizer-watermark false`. Assert only `OptimizerWatermarkEnabled: false` is sent.
3. Round-trip an Optimizer-rich `pull-zone get` response — no panics, all 24 fields surface in the JSON output.
4. Watermark position kebab-case → int mapping for all five positions.
5. `pull-zone get` against a Pull Zone that has Optimizer disabled — Optimizer fields all serialise as `null` / are absent (depending on bunny's response shape; capture and pin behaviour).

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| Add 24 fields to `PullZone` + writable subset to `UpdatePullZone` | Small (mechanical) |
| Watermark-position repr-enum + helper | Small |
| CLI flags + grouped help heading | Medium (many flags, but all the same shape) |
| Wiremock tests (round-trip + per-flag) | Medium |
| Help text / examples / quirks doc | Small |
| **Total** | **Medium** |

## Related

- Gap analysis (this conversation, 2026-05-09)
- Pull Zone schema: <https://core-api-public-docs.b-cdn.net/docs/v3/public.json>
- Optimizer docs: <https://docs.bunny.net/docs/optimizer>
- [[iteration-19-pullzone-bugfixes]] (`deserialize_repr_option` helper, ArgGroup pattern)
- [[api/bunny-api-quirks]]
- [[decision-log]]
