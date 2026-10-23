---
title: OpenAPI spec drift since the July 2026 snapshot (2026-10-23)
type: research
date: 2026-10-23
status: completed
origin: fresh download of all published bunny.net OpenAPI specs, diffed structurally against specs/ (last refreshed in iter-66, commit 339adce, 2026-07-10)
tags:
  - openapi
  - api-drift
  - gap-analysis
  - planning
---

# OpenAPI spec drift since the July 2026 snapshot (2026-10-23)

Fresh downloads of every published bunny.net OpenAPI spec, diffed
structurally (operations, parameters, request/response refs, schema
properties, enum values) against the `specs/` snapshot that
[[iterations/iteration-66-spec-refresh-drift-fixes]] committed on 2026-07-10
(commit `339adce`). Spec URLs come from
[[research/api-coverage-gap-analysis-2026-07]] and the `## OpenAPI Specs`
section of `https://bunny.net/docs/llms.txt` (`docs.bunny.net` now
redirects to `bunny.net/docs`). Each finding was checked against the
client types in `crates/bunny-net-api/src/`.

`specs/` was **not** updated in this pass. In iter-66 the refresh was step 1
of a drift-fix iteration, and this note follows that precedent: the next
drift-fix iteration commits the fresh specs together with the type fixes.
See [[backlog/specs-snapshot-refresh-2026-10]].

## 1. Spec sources: old vs new

| Spec | Canonical URL | July snapshot | Today | Change |
|---|---|---|---|---|
| core-platform | core-api-public-docs.b-cdn.net/docs/v3/public.json | 1.0.0, 93 ops | 1.0.0, 111 ops | **+18 ops** (Load Balancer ×16, external HTTP cert ×2) plus pricing; new fields/enums |
| shield | api.bunny.net/shield/docs/v1/swagger.json | v1, 61 ops | v1, 65 ops | +4 ops; **4 enum widenings**; 1 request field removed |
| stream | video.bunnycdn.com/openapi/bunnynet-video-api.public.json | 1.5.6, 28 ops | **1.6.12**, 29 ops | +1 op; cleanup response gained a payload; error codes documented |
| magic-containers | api-mc.opsbunny.net/docs/public/swagger.json | 1.0, 52 ops | 1.0, 54 ops | +2 ops; `ContainerEndpoint.id`; **VolumeStatus widened** |
| database | api.bunny.net/database/docs/private/api.json | 0.0.130, 34 ops | **0.0.144**, 34 ops | `per_page` default 0→50, `0` rejected |
| edge-scripting | core-api-public-docs.b-cdn.net/docs/v3/compute.json | 1.0.0, 23 ops | 1.0.0, 23 ops | security-scope list gained `ScopedApiKey` only |
| storage | bunny.net/docs/openapi/bunnynet-edge-storage-api.json | 1.0.0 | 1.0.0 | byte-identical |
| cdn-logging | logging.bunnycdn.com/docs/all/swagger.json | v2 | v2 | byte-identical |
| origin-errors | bunny.net/docs/openapi/origin-errors-spec.json | 1.0.0 | 1.0.0 | byte-identical |
| **partner-portal** | core-api-public-docs.b-cdn.net/docs/v3/PartnerPortal.json | — | 1.0.0, 13 ops | **NEW spec**, not in `specs/` |

`info.version` is useless as a change signal for core, shield, MC and
compute: it has been `1.0.0` / `v1` / `1.0` across substantive changes.
Diff the content.

## 2. Findings ranked by impact

### Breaking: deserialization failures on live data

Each of these is a closed enum (`Deserialize_repr` / string enum, no
fallback) on a **non-`deserialize_repr_option`** response field. One
unknown value fails the whole response, and with it the list or get
command. This is the same failure mode iter-19 fixed for `OriginType`.

1. **Shield `DDoSShieldSensitivity` gained `5 = ChallengeInteractive`.**
   `ShieldZoneResponse.d_do_s_shield_sensitivity` (`shield/types.rs:673`)
   uses plain `#[serde(default)]`, so `shield zone get/list` fails for any
   zone set to the new level. The existing naming is also wrong: hoppy
   calls `4` `VeryHigh` and `0` `Disabled`, but the spec has `4 = Challenge`
   and `0 = Off`, and has done since July.
   → [[backlog/shield-ddos-sensitivity-challenge-interactive]]
2. **Shield `WafRuleOperatorType` gained `13 = IPMATCH`.** It is
   non-optional on `WafChainedRuleCondition`, `WafRuleConfiguration` and
   `RateLimitRuleConfiguration` (`shield/types.rs:778,790,867`). Listing
   custom WAF or rate-limit rules fails once any rule uses IP matching. The
   variant names are also wrong against the spec: hoppy `Eq=0` means
   `BEGINSWITH`, `Regex=6` means `GE`, and so on. The CLI's operator names
   send the wrong operator today.
   → [[backlog/shield-waf-operator-ipmatch]]
3. **Shield `BotCategorizationAction` gained `3 = Ignore`.**
   `BotCategorizationDetails.action` (`shield/types.rs:2055`) is
   non-optional, so `shield bot-categorization list` fails once any bot is
   set to Ignore. The spec now splits the types: per-bot actions use
   `BotCategorizationAction` (0–3), and category-level actions plus the
   `PUT …/categories/{category}` body use the new `BotCategoryAction` (0–2).
   → [[backlog/shield-bot-categorization-ignore-action]]
4. **Magic Containers `VolumeStatus` gained `maintenance`.**
   `VolumeInstance.status: VolumeInstanceStatus` (`containers/types.rs:981`)
   is a closed string enum, so `container volume list/get` fails while bunny
   has any volume in maintenance. Bunny controls that state; the user
   cannot avoid it.
   → [[backlog/container-volume-status-maintenance]]

### Likely breaking or silent data loss: needs a live check

5. **Shield `ShieldZoneRequest.dDoSExecutionMode` was removed** from the
   request schema. It is still in the response. hoppy still sends it from
   `shield zone update --ddos-execution-mode` (`shield/types.rs`
   `ShieldZoneRequest.d_do_s_execution_mode`,
   `hoppy-cli/src/commands/shield.rs:770`). The API either ignores it now
   (the flag silently does nothing) or rejects it.
   → [[backlog/shield-ddos-execution-mode-request-removed]]
6. **`StorageRegionModel` was reshaped**: `Id` and `Url` were removed, and
   `Code`, `Endpoint`, `S3Endpoint`, `Continent`, `Latitude`, `Longitude`,
   `PricePerGigabyte`, `Tier`, `IsMainRegionSupported` and `IsS3Supported`
   were added. `core::StorageRegion` (`core/types.rs:4711`) models only
   `id`, `name` and `url`, all optional, so `storage-zone regions` would
   print empty code and URL columns without any error. The checked-in
   fixture `fixtures/core/storagezone_regions.json` still has the July
   shape.
   → [[backlog/storage-zone-regions-shape-change]]
7. **The pull-zone `EdgeScriptExecutionPhase` enum is misnamed and was
   widened.** The spec schema `ExecutionPhase` is `0 = Cache`,
   `1 = LoadBalancer`, `2 = PreCache` (new). hoppy models it as
   `OnRequest = 0` / `OnResponse = 1` (`core/types.rs:534`). Reads of `2`
   fall back to `None` (the field uses `deserialize_repr_option`), and the
   CLI cannot set it. The naming has been wrong since iter-74.
   → [[backlog/pull-zone-execution-phase-naming]]

### Missing feature: partial coverage of changed endpoints

8. **The edge-rule `ActionType` gained 35–37**: `UseLoadBalancer`,
   `ForceLoadBalancerStickySessionMode`, `StripCookiesClientToOrigin`.
   `EdgeRuleActionType` (`core/types.rs:61`) stops at 34. Reads degrade to
   `None` because the field uses `deserialize_repr_option`, so `edge-rule
   list` shows no action. The CLI cannot create these rules.
   → [[backlog/edge-rule-action-types-35-37]]
9. **The Stream `POST …/resolutions/cleanup` response** changed from
   `StatusModel` to `StatusModelOfResolutionsCleanupModel`, which carries a
   full plan (resolutions to delete, storage objects, playlist changes).
   `cleanup_video_resolutions` returns `StatusMessage`
   (`stream/client.rs:584`) and discards it. As a result, `--dry-run` (which
   maps to the server-side `dryRun=true`) shows nothing useful.
   → [[backlog/stream-resolutions-cleanup-report-dropped]]
10. **Magic Containers `ContainerEndpoint` now has a required `id`.**
    `containers::ContainerEndpoint` (`containers/types.rs:781`) has no `id`
    field; only `EndpointListItem` does. As a result, `container app get`
    cannot show endpoint ids, and users have to run `endpoint list` to
    address an endpoint.
    → [[backlog/container-endpoint-id-missing]]
11. **New pull-zone settings** on `PullZoneModel`, `PullZoneAddModel` and
    `PullZoneSettingsModel`: `IpFamilyPolicy` (IPv4/dual/IPv6),
    `TlsSecurityLevel` (Legacy/Compatible/ModernOnly), `LoadBalancerId`,
    `OptimizerHtmlToMarkdown`, `OptimizerLlmsTxt` and
    `OptimizerTunnelActiveProbingEnabled`. None of them are modeled in
    `core::PullZone` or `UpdatePullZone`.
    → [[backlog/pull-zone-new-settings-2026-10]]

### Missing feature: new endpoints and services

12. **Load Balancer API (core, 16 ops)**: profiles, origin groups, origins,
    statistics, usage and attached pull zones under `/loadbalancer`. hoppy
    has only `OriginType::LoadBalancer = 3`.
    → [[backlog/load-balancer-api-missing]]
13. **New Shield surface**: `POST /shield/event-logs/{id}/search` (filter,
    group, paginate), `POST …/export` (CSV),
    `GET /shield/shield-zone/defaults`, and
    `POST /shield/shield-zone/under-attack`. `CreateShieldZoneRequest` also
    gained `accessLists`, `botDetectionExecutionMode`, `csamScanningMode` and
    `antivirusScanningMode`. WAF conditions gained `isNegated` and
    `isRegexVariable`, and `WafMappedEnum.valueConstraint` was added.
    → [[backlog/shield-event-logs-search-and-zone-defaults]]
14. **External HTTP certificate flow**:
    `POST /pullzone/requestExternalHttpCertificate` and
    `…/completeExternalHttpCertificate`. This is the HTTP-01 counterpart of
    the DNS flow hoppy already wraps (`core/client.rs:356`).
    → [[backlog/pull-zone-external-http-certificate]]
15. **DNS read filters**: `GET /dnszone/{zoneId}/records` gained `search` and
    `type`, and `GET /dnszone/{id}` gained `view` (`Full`/`Lite`).
    `list_dns_zone_records` (`core/client.rs:858`) sends only `page` and
    `perPage`, and `get_dns_zone` sends no query parameters.
    → [[backlog/dns-record-list-filters]]
16. **Magic Containers registries**: the new `bunnyRegistry` registry type,
    `ContainerRegistry.isPlatformManaged`, `POST /registries/images/find`
    and `POST /registries/tags/paged`. Separately, `RegistryType` serializes
    as `DockerHub`/`GitHub` while the spec says `dockerHub`/`gitHub`; this
    mismatch predates the July snapshot and needs a live check.
    → [[backlog/container-registry-bunny-registry]]
17. **Magic Containers overview usage**: `Overview.limits`, the per-pod
    `cpuUsageCores`, `ramUsageBytes`, `outboundBandwidthBytesPerSecond` and
    `volumes`, and the per-pod caps on `UserLimits`. None are modeled.
    → [[backlog/container-overview-usage-fields]]
18. **Stream**: `GET /library/{id}/statistics/encoding` (new),
    `CollectionModel.liveStreamCount`, the Widevine fields and `pullZoneId`
    on `VideoPlayDataModel`, and the oEmbed thumbnail sizes.
    `CaptionModelAdd.srclang` is now deprecated in the body (the path
    parameter carries it), which matters for `stream/types.rs:55`.
    → [[backlog/stream-encoding-statistics]]
19. **Partner Portal API (new spec, 13 ops)**: reseller customer management,
    billing, pricing and audit under `/partner/*`. Relevant only to partner
    accounts.
    → [[backlog/partner-portal-api-missing]]
20. **Billing and pricing**: the AI Gateway and Load Balancer charge fields on
    `BillingModel`, and `GET /v1/pricing/{source}/{resourceId}` (price
    estimation).
    → [[backlog/billing-pricing-estimation-2026-10]]
21. **Core miscellaneous fields**: `VideoLibraryModel.GeoBlocking`
    (read/update), `LiveStreamCount` and `TotalCount`, and
    `StorageZoneModel.S3Hostname`.
    → [[backlog/core-misc-new-fields-2026-10]]

### Cosmetic or low

22. **Database `GET /v2/databases`**: `per_page` now defaults to 50, and `0`
    is rejected with a new 400 response. `db list --auto` is unaffected
    because it passes `AUTO_PER_PAGE`. `db list --per-page 0` would now fail
    with a 400 instead of a clean CLI error.
    → [[backlog/db-list-per-page-zero-rejected]]
23. **No backlog item**: the additional documented error codes (409 on DNS
    deletes and MC restart, 429 on external-cert calls, many 401/403/404 on
    Stream). hoppy maps errors by status generically. `ScopedApiKey` was
    added to the compute security scopes, which documents a new key kind
    with no client impact.

## 3. Specific questions

- **Is there a `GET /pullzone` parameter that includes Magic Containers
  (origin type 5) zones?** No. The parameter list is unchanged since July:
  `page`, `perPage`, `search`, `includeCertificate`. The spec has no
  origin-type filter and no `includeHidden` or `includeMagicContainers`
  flag. `PullZoneOriginType` documents `5 = MagicContainers` (and
  `1 = DnsAccelerate` and `6 = PushZone`, which `core::OriginType` still
  lacks; they read as `None` through `deserialize_repr_option`). If MC
  pull zones are missing from `pull-zone list`, the spec gives no
  documented way to include them. Confirm against the live API before
  adding a client-side workaround.
- **Does the Magic Containers endpoint schema include `id`?** Yes, as of
  this snapshot. `ContainerEndpoint` now lists `id` (string) as
  **required**. The July snapshot had it only on the list-item schema. See
  finding 10.

## 4. Recommendation

Run one drift-fix iteration (the iter-66 pattern) that:

1. commits the refreshed `specs/` and adds `specs/partner-portal.json`;
2. fixes findings 1–4 by switching every remaining closed response enum to
   the `deserialize_repr_option` / `#[serde(other)] Unknown` fallback. Do
   this as an audit of all response enums, not just the four that broke
   this time. Fix the shield enum naming (DDoS sensitivity, WAF operator)
   in the same change;
3. dogfoods findings 5–7 (shield DDoS execution mode, storage regions
   shape, execution phase) against the live API before changing payloads.

Findings 8–21 are feature work for later iterations. Load Balancer (12) and
the shield event-log search (13) are the largest gaps by user value.

## 5. Method

- Downloads went to a fresh scratch directory and were never executed. A
  structural diff script (in a separate directory, `python3 -I`) compared
  operations, parameters, body and response `$ref`s, schema properties,
  `required` and enums. Inline enums were checked by hand via `jq`.
- `storage`, `logging` and `origin-errors` are byte-identical to `specs/`.
- `docs.bunny.net/openapi/*.json` redirects to `bunny.net/docs/openapi/*`.
  The Mintlify `api-reference/*/openapi.json` mirrors still differ from
  the canonical files, so keep pulling from the canonical URLs.
