---
title: Bunny.net API coverage gap analysis (July 2026)
type: research
date: 2026-07-03
status: active
origin: full deep dive 2026-07-03 — fresh specs pulled from docs.bunny.net, all 298 operations compared against hoppy CLI v0.3.0 (main @ 31f3757)
tags:
  - api-coverage
  - gap-analysis
  - planning
---

# Bunny.net API coverage gap analysis (July 2026)

Full audit of every published Bunny.net OpenAPI operation against the hoppy
CLI surface (287 commands, fresh `--help` dump) and the `bunny-net-api`
client source. Detail reports with per-endpoint and per-flag tables live in
`research/api-coverage-2026-07/`:
[[research/api-coverage-2026-07/pullzone-misc|Pull zones + misc]],
[[research/api-coverage-2026-07/dns|DNS]],
[[research/api-coverage-2026-07/storage|Storage]],
[[research/api-coverage-2026-07/stream|Stream + Video Library]],
[[research/api-coverage-2026-07/script|Edge Scripting]],
[[research/api-coverage-2026-07/shield|Shield]],
[[research/api-coverage-2026-07/database|Database]],
[[research/api-coverage-2026-07/containers|Magic Containers]].

Complements (does not replace) the iter-43 struct-level audit in
[[research/spec-coverage/README|research/spec-coverage/]] (2026-05-31): that
pass audited response-struct/write-payload **fields** against the March
specs; this one audits **endpoint and flag coverage** against the July
specs. The coverage claims here are the current source of truth.

## 1. Spec freshness — what Bunny publishes today

Bunny publishes exactly **9 OpenAPI specs** (index: <https://docs.bunny.net/openapi>
and the `## OpenAPI Specs` section of <https://docs.bunny.net/llms.txt>).
`specs/` has been refreshed from the canonical URLs (uncommitted as of this
writing); three services had **no local spec at all** before this pass.

| Service | Canonical URL | Local → fresh | Delta |
|---|---|---|---|
| core-platform | core-api-public-docs.b-cdn.net/docs/v3/public.json | 87 → 93 ops | +6 (see below) |
| shield | api.bunny.net/shield/docs/v1/swagger.json | 50 → 61 ops | +12 / −1 (API Guardian rework, bot categorization, custom pages, overages) |
| stream | video.bunnycdn.com/openapi/bunnynet-video-api.public.json | 28 → 28 | schema-only bump v1.0.0 → v1.5.6 |
| edge-scripting | core-api-public-docs.b-cdn.net/docs/v3/compute.json | 23 → 23 | unchanged |
| database | api.bunny.net/database/docs/private/api.json | 34 → 34 | unchanged (v0.0.130) |
| storage | docs.bunny.net/openapi/bunnynet-edge-storage-api.json | 4 → 4 | +2 regional server entries |
| **magic-containers** | api-mc.opsbunny.net/docs/public/swagger.json | **NEW** → 52 ops | server base `https://api.bunny.net/mc` |
| **cdn-logging** | logging.bunnycdn.com/docs/all/swagger.json | **NEW** → 2 ops | pull-zone access log retrieval |
| **origin-errors** | docs.bunny.net/openapi/origin-errors-spec.json | **NEW** → 1 op | origin error logs |

Core-platform's 6 new ops: `GET /dnszone/{zoneId}/records`, `GET /pullzone/count`,
`GET /storagezone/regions`, `GET /storagezone/{id}/statistics/egress`,
`POST /pullzone/requestExternalDnsCertificate`, `POST /pullzone/completeExternalDnsCertificate`.
None are implemented in hoppy.

Caveats from the online research:

- The `docs.bunny.net/openapi/*.json` Mintlify mirrors are **subsets** for
  core/shield — always pull from the canonical URLs above. The Mintlify
  stream mirror contains 13 live-streaming endpoints that are stale/removed —
  do not implement them.
- Docs-only surfaces with no spec: Token Authentication URL signing
  (client-side crypto, no endpoint), Stream **TUS resumable uploads**
  (separate upload protocol), Optimizer (URL-param based), Stream webhooks
  (configured via video-library settings).

## 2. Coverage scorecard

298 published operations. **174 covered (58%), 53 partial (18%), 71 missing (24%).**
"Partial" = command exists but drops documented parameters/body fields.

| Domain | Ops | Covered | Partial | Missing |
|---|---|---|---|---|
| Magic Containers | 52 | 39 | 8 | 5 |
| Database | 34 | 28 | 3 | 3 |
| Edge Scripting | 23 | 18 | 3 | 2 |
| Stream API (video.bunnycdn.com) | 28 | 17 | 8 | 3 |
| DNS (core, incl. 1 new) | 18 | 12 | 4 | 2 |
| Pull zones (core, incl. 3 new) | 30 | 18 | 6 | 6 |
| Edge Storage (file ops) | 4 | 3 | 1 | 0 |
| Shield (fresh spec, 61) | 61 | 31 | 12 | 18 |
| Storage zones (core, incl. 2 new) | 11 | 2 | 4 | 5 |
| Video Library (core) | 20 | 5 | 2 | 13 |
| Core misc (billing/user/apikey/…) | 14 | 1 | 2 | 11 |
| CDN Logging | 2 | 0 | 0 | 2 |
| Origin Errors | 1 | 0 | 0 | 1 |

Strongest: Magic Containers, Database, Edge Scripting (the newest modules).
Weakest: Video Library settings, core misc (account/billing), Shield's new
surface, and the two log-retrieval services (0%).

## 3. Correctness drift — fix before adding features

These are not gaps; they are places where hoppy disagrees with the current API:

1. **`db fork` payload drift** — spec requires `{slug, date}` (point-in-time
   fork); client sends `{slug, group}` (`database/types.rs:225`). PITR forks
   unreachable; payload may start failing on spec enforcement.
2. **`shield api-guardian upload`** targets `POST …/api-guardian`, which the
   fresh spec **removed** — replaced by `POST/PATCH …/api-guardian/spec`
   (plus new `GET …/api-guardian/enums`).
3. **`db config optimal`** omits the spec-required `cdn_server_token`; the
   hidden `optimal_single` stub never calls the API at all.
4. **`stream caption add`** sends raw SRT where docs indicate base64 —
   needs dogfood verification.
5. **Storage regional hosts** — client builds `{region}.bunnycdn.com`;
   documented hosts are `{region}.storage.bunnycdn.com`. Only the default
   host is exercised by fixtures — needs dogfood verification per region.
6. **`storage download` buffers whole files in memory** — violates the
   project streaming rule (CLAUDE.md) for arbitrarily large blobs.
7. **Reverse drift (CLI sends undocumented fields)**: shield `--ddos-*`
   update flags and `ShieldZoneRequest` fields like `blockVpn`/`blockTor`
   have no counterpart in the fresh spec's `shieldZone` schema.

## 4. Top gaps by theme

**Security / credential rotation (small, high value, all missing):**
storage-zone `resetPassword` + `resetReadOnlyPassword`, video-library
`resetApiKey` + `resetReadOnlyApiKey`, pull-zone `resetSecurityKey`,
`storage upload --checksum` (client already plumbed).

**Update commands that can't update:** shield `waf update-rule` /
`rate-limit update` can only rename (conditions/actions/thresholds
immutable → delete+recreate); `dns record update` lacks `--port/--flags/--tag`
(SRV/CAA updates lossy) and forces re-specifying `--type/--value`; no
`db group update` / `db v2 update` at all (client payloads exist but are
dead code, and model non-spec fields).

**Server-side filters & pagination silently dropped:** shield's three
paginated lists (zones, custom rules, rate limits) always return page 1;
`script list` lacks `--type/--integration-id/--include-linked-pullzones`;
`hoppy statistics` drops 9 of 13 query params; `purge` drops `exactPath`
(single-URL purge semantics uncontrollable) and `async`; stream `video upload`
drops all 10 per-upload encoding/transcription params; `db versions` lacks
`--older-than/--newer-than`; shield `metrics detailed` ignores
`StartDate/EndDate/Resolution`.

**Whole missing sub-surfaces:**

- Video Library settings: update reaches only 4 body fields — resolutions,
  codecs, player/webhook/DRM/transcription config, 4 referrer allow/block
  endpoints, watermark upload/delete, languages list — all unreachable.
- Core misc: API keys, billing (summary/invoices/payment requests/PDFs),
  region+country reference data, global search, user audit log — 11 of 14
  ops missing.
- Shield fresh surface: bot categorization (3), custom block/challenge
  pages (3), overages metrics, API Guardian metrics (2) — all new, all missing.
- CDN Logging + Origin Errors: both services 100% missing (3 ops total,
  trivially small modules).
- DNS smart features: `SmartRoutingType`, monitor/latency fields, geo
  lat/long, linked-record fields (`--type PullZone/Script` accepted but
  non-functional), record `Disabled` toggle, zone `LogAnonymizationType`.
- Pull-zone body: 21 unreachable props on update — error-page group (5),
  preloading-screen group (7), `EdgeScriptId`/`MiddlewareScriptId` (can't
  wire `hoppy script` output to a zone), MagicContainers origin fields.
- Magic Containers polish: `volumes`/`volumeMounts` unexposed (volume
  subcommands only usable on dashboard-created volumes), container probes /
  entrypoint / imagePullPolicy, endpoint `isSslEnabled`/`stickySessions`/
  `protocols`/`pullZoneId`, `registry images` (client method exists,
  no command), app summary / nodes / image-config endpoints.
- Stream: TUS resumable uploads (docs-only protocol), OEmbed, play,
  heatmap endpoints.

## 5. Proposed iteration plan

Ordered by (correctness first, then value/effort). One iteration = one
branch = one PR, per project convention.

- [ ] **P0 — iter-66 “spec refresh & drift fixes”**: commit refreshed
  `specs/` (+3 new specs); fix `db fork` payload; rework API Guardian to
  `/spec` endpoints + `enums`; add `cdn_server_token` to db optimal (unstub
  `optimal_single`); make `storage download` stream to disk; dogfood-verify
  caption base64 + regional storage hosts; refresh magic-containers KB notes
  (3 endpoints absent from notes).
- [ ] **P1 — iter-67 “credential rotation”**: the six rotation/safety
  commands + `--checksum` + `storage-zone delete --delete-linked-pull-zones`
  + directory-delete semantics for `storage rm`.
- [ ] **P1 — iter-68 “updates that update”**: full-field shield
  `waf update-rule`/`rate-limit update` (+ `--config-json` escape hatch for
  nested conditions), `dns record update` parity + `--disabled`,
  `db group update`, `db v2 update` (fix non-spec client payloads).
- [ ] **P2 — iter-69 “filters & pagination sweep”**: everything in the
  filters theme above (shield pagination, script list filters +
  `--load-latest`, statistics params, purge `exactPath`/`async`,
  storage-zone `--include-deleted`, db versions windowing, shield metrics
  time range, stream per-upload params).
- [ ] **P2 — iter-70 “log retrieval services”**: new feature-gated
  `logging` + `origin_errors` modules in bunny-net-api; `hoppy logs pull-zone`
  and `hoppy logs origin-errors` (or similar) commands. Tiny but 100%-new
  coverage; streaming download path.
- [ ] **P2 — iter-71 “DNS completeness”**: smart routing/monitoring/geo
  fields, linked-record wiring, `GET /dnszone/{id}/records`, zone
  `LogAnonymizationType`, `checkavailability` (dns + pullzone + storagezone
  in one go), `pullzone count`, storagezone `regions` + egress statistics.
- [ ] **P3 — iter-72 “shield new surface”**: bot categorization, custom
  pages, overages + API Guardian metrics; drop/flag the undocumented
  `--ddos-*` fields per drift note.
- [ ] **P3 — iter-73 “video library settings”**: full library update
  surface, referrer ops, watermark upload/delete, languages, OEmbed/play/
  heatmap; video update `chapters`/`moments`/`metaTags`.
- [ ] **P3 — iter-74 “pull-zone body completeness”**: error pages,
  preloading screen, EdgeScriptId/MiddlewareScriptId, MagicContainers
  origins, `updatePrivateKeyType`, external-DNS certificate flow.
- [ ] **P4 — iter-75 “account & billing”**: apikey list, billing
  summary/invoices/PDFs, region/country reference, global search, audit log.
  (`closeaccount` deliberately excluded — destructive, low CLI value.)
- [ ] **P4 — iter-76 “containers polish”**: volumes/volumeMounts, probes/
  entrypoint/imagePullPolicy, endpoint SSL/sticky/protocols, `registry images`,
  app summary + nodes + image-config.
- [ ] **P4 — iter-77 “stream TUS resumable upload”**: TUS protocol client
  (bigger feature; unlocks reliable large uploads + the per-upload params
  from iter-69 apply here too).

## 6. Method note

Fresh `--help` tree (287 commands) dumped from a release build of main
@ 31f3757; every spec operation's params/body compared against clap flags,
falling back to `hoppy-cli` and `bunny-net-api` source where help text was
ambiguous. Endpoint inventories were machine-extracted from the specs, so
"missing" claims are exhaustive per spec; flag-level "partial" claims were
verified by agents against source but spot-checking during implementation
is still advised.
