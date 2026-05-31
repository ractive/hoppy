---
title: "Pull-zone update gap — severity buckets"
type: research
date: 2026-05-31
tags:
  - audit
  - openapi
  - spec-coverage
  - pull-zone
  - planning
---

# Pull-zone update gap — severity buckets

Hand-authored companion to the regenerable [[pull-zone]] audit. Lives
in its own file because `run-spec-coverage-audit.sh` overwrites the
mechanical per-resource report on every run; this file is preserved.

Categorises every field the update payload (`PullZoneSettingsModel`)
is missing into severity / theme buckets that map cleanly to iteration
scoping. A single field may be load-bearing for more than one bucket;
choose the dominant one.

### 🔴 Security / compliance

- `EnableTLS1`, `EnableTLS1_1` — deprecated TLS toggles (PCI/SOC2).
- `EnableAutoSSL`, `DisableLetsEncrypt` — cert provisioning controls.
- `VerifyOriginSSL` — origin TLS chain verification.
- `EnableAccessControlOriginHeader`,
  `AccessControlOriginHeaderExtensions` — CORS exposure.
- `ZoneSecurityIncludeHashRemoteIP` — token-auth IP binding.
- `AWSSigningEnabled`, `AWSSigningKey`, `AWSSigningSecret`,
  `AWSSigningRegionName` — SigV4 to private S3 origins (secrets).
- `LoggingIPAnonymizationEnabled`, `LogAnonymizationType` — GDPR knobs.

### 🟠 Firewall / blocking

- `BlockedCountries`, `BudgetRedirectedCountries`.
- `BlockedIps`, `AllowedReferrers`, `BlockedReferrers`.
- `BlockNoneReferrer`, `BlockPostRequests`, `BlockRootPathAccess`.
- `DisableCookies`.
- `ShieldDDosProtectionEnabled`, `ShieldDDosProtectionType`.
- `BurstSize`, `RequestLimit`, `LimitRateAfter`, `LimitRatePerSecond`,
  `ConnectionLimitPerIPCount` — rate limiting.
- `MaxWebSocketConnections`.

### 🟡 Performance / caching

- `EnableCacheSlice`, `EnableSmartCache`, `EnableSafeHop`.
- `IgnoreQueryStrings`, `EnableQueryStringOrdering`,
  `QueryStringVaryParameters`, `CookieVaryParameters`.
- `UseStaleWhileUpdating`, `UseStaleWhileOffline`, `UseBackgroundUpdate`.
- `CacheControlMaxAgeOverride`, `CacheControlPublicMaxAgeOverride`,
  `CacheControlBrowserMaxAgeOverride`.
- `CacheErrorResponses`.
- `PermaCacheStorageZoneId`, `PermaCacheType`.

### 🟡 Vary headers

- `EnableWebpVary`, `EnableAvifVary`, `EnableCookieVary`,
  `EnableCountryCodeVary`, `EnableCountryStateCodeVary`,
  `EnableHostnameVary`, `EnableMobileVary`.

### 🟢 Routing / origin

- `OriginHostHeader`, `AddHostHeader`, `AddCanonicalHeader`.
- `DnsOriginPort`, `DnsOriginScheme`.
- `FollowRedirects`.
- `OriginConnectTimeout`, `OriginResponseTimeout`, `OriginRetries`,
  `OriginRetry5XXResponses`, `OriginRetryConnectionTimeout`,
  `OriginRetryDelay`, `OriginRetryResponseTimeout`.
- `EnableOriginShield`, `OriginShieldEnableConcurrencyLimit`,
  `OriginShieldMaxConcurrentRequests`,
  `OriginShieldMaxQueuedRequests`, `OriginShieldQueueMaxWaitTime`,
  `OriginShieldZoneCode`.
- `EnableRequestCoalescing`, `RequestCoalescingTimeout`.
- `RoutingFilters`.
- `StickySessionType`, `StickySessionCookieName`,
  `StickySessionClientHeaders`.
- `Type` — pull zone tier (Standard / Volume).

### 🔵 Observability / logging

- `EnableLogging`.
- `LogFormat`, `LogForwardingFormat`.

### 🟣 Edge scripting / containers / AI

- `EdgeScriptId`, `EdgeScriptExecutionPhase`, `MiddlewareScriptId`.
- `MagicContainersAppId`, `MagicContainersEndpointId`.
- `EnableBunnyImageAi`, `BunnyAiImageBlueprints`.

### ⚪ Niche / cosmetic

- `EnableWebSockets`.
- `ErrorPageCustomCode`, `ErrorPageEnableCustomCode`,
  `ErrorPageEnableStatuspageWidget`, `ErrorPageStatuspageCode`,
  `ErrorPageWhitelabel`.
- `PreloadingScreenEnabled`, `PreloadingScreenCode`,
  `PreloadingScreenCodeEnabled`, `PreloadingScreenDelay`,
  `PreloadingScreenLogoUrl`, `PreloadingScreenShowOnFirstVisit`,
  `PreloadingScreenTheme`.

## Recommended next-iteration scoping

The 🔴 **security / compliance** bucket is the only one that
*blocks* PCI/SOC2 use cases. It is small (~13 fields, all booleans
or short strings/enums) and ships cleanly as one surgical PR
(struct fields → update payload → CLI flags → help text → snapshot
refresh).

After that, three natural bundles:

1. **Vary + caching** (~16 fields) — biggest *user value* per PR;
   tightly themed; mostly booleans.
2. **Origin + routing** (~20 fields) — denser and mostly numbers,
   so help-text + validation work dominates.
3. **Everything else** — niche pages, error pages, preloading.
   Lowest priority; consider triaging into the backlog instead of
   shipping in bulk.

Edge-scripting / containers / AI overlap with iter-21/iter-26 surface
area — flag them for the relevant resource owners rather than
bundling into a pull-zone PR.
