---
title: API shape drift found by the 2026-07-10 fixture-refresh sweep
type: research
date: 2026-07-10
status: active
origin: fixture-refresh recording sweep 2026-07-10 (post iter-66..77)
tags:
  - api-coverage
  - fixtures
  - shape-drift
---

# API shape drift found by the 2026-07-10 fixture-refresh sweep

A full `HOPPY_RECORD_DIR` recording sweep against the test account was
compared to the checked-in fixtures (`fixture-refresh` + per-file key diff).
Most refreshed fixtures were **reverted** because the wiremock tests pin
fixture values — but the key-diff surfaced real API shape drift the client
doesn't model yet. This is planning input for upcoming iterations.

## New fields per surface (live response vs checked-in fixture)

### DNS records (`dnsrecord_add`, `dnszone_records_paginated`)

`AccelerationStatus`, `AutoSslIssuance`, `EnviromentalVariables` (sic — API
typo), `GeolocationInfo`, `GeolocationLatitude`, `GeolocationLongitude`,
`IPGeoLocationInfo`, `LatencyZone`, `MonitorStatus`, `MonitorType`,
`SmartRoutingType`, plus record-level `Id` on add-response.

### Pull zones (`pullzone_get`)

`CacheKeyHeaders`, `IpFamilyPolicy`.

### Video libraries (core API — biggest gap, ~98 new fields)

The checked-in fixtures are minimal; the live response includes full DRM,
encoding and player config: `AppleFairPlayDrm.*`,
`GoogleWidevineDrm.*`, `Bitrate240p`–`Bitrate2160p`, `AllowEarlyPlay`,
`AllowedReferrers`, `ApiAccessKey`, transcoding/player settings, and more.
`videolibrary_get` also gained `ReplicationRegions` entries.

### Shield zones (`shield_zone_get`, `shield_zones_list`)

`learningModeUntil`, `requestBodyLoggingEnabled`, `wafCustomRuleOrder`,
`wafDisabledRules`, `wafEngineConfig`, `dDoSChallengeWindow`, and siblings
(~15 fields per zone object).

### Shield access lists (`access_lists_get`)

Custom lists gained `category`, `lastUpdated`, `requiredPlan`,
`updateFrequency`.

### Stream collections (`collection_create`, `collection_list_paginated`)

Live v2 responses are camelCase (`guid`, `name`, `previewImageUrls`,
`previewVideoIds`, `totalSize`, `items`, `currentPage`) where the
checked-in fixtures use PascalCase — casing drift worth verifying against
the client's serde renames.

## Non-drift observations

- **Plan-tier error envelopes**: shield `bot_detection_get`,
  `metrics_rate_limits`, `rate_limit_rule_create`, `access_list_create`
  recordings captured `error`/`errorResponse` envelopes (test-account plan
  doesn't include those features) — recordings from this account can't
  refresh those fixtures. Matches the plan-tier guards added in iter-66..77.
- **Statistics/metrics fixtures**: chart payloads are date-keyed maps that
  change daily; recording-based refresh will always churn them. Tests
  rightly pin hand-authored dates — exclude these from refresh sweeps.

## Follow-up candidates

- [ ] Iteration: video library settings completeness (DRM, bitrates,
      referrers) — largest gap
- [ ] Iteration: DNS record monitor/geolocation/smart-routing fields
- [ ] Iteration: shield zone new config fields
- [ ] Check stream collection casing handling in the client
- [x] Teach `fixture-refresh` to skip date-keyed chart fixtures

## Related

- [[dogfooding/dogfooding-playbook]]
- [[research/api-coverage-gap-analysis-2026-07]]
- [[backlog/fixture-git-history-dead-deployment-keys]]
