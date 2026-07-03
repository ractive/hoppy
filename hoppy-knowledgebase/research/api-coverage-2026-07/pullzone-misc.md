---
title: "Gap report: Pull Zones, Purge, Statistics, Account misc"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - pull-zone
  - purge
  - statistics
  - billing
---

# Pull Zone + core misc gap report

Domain: Pull Zones (27 ops, `inventories/core-split/pullzone.txt`) + core misc (14 ops, `inventories/core-split/misc.txt`).
Sources: `crates/hoppy-cli/src/commands/{pull_zone,purge,statistics,auth}.rs`, `crates/bunny-net-api/src/core/{client,types}.rs`, clap help dumps.

## 1. Endpoint coverage

### Pull Zones (27 ops)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /pullzone | `pull-zone list` | partial | `page`, `perPage`, `search` covered (+ CLI-side `--all`); `includeCertificate` MISSING (also absent in client) |
| POST /pullzone | `pull-zone create` | partial | Only `Name`, `OriginUrl`, `StorageZoneId`, `Type` (`--zone-tier`) exposed. Deliberate create-minimal design; everything else set via `update`. Client struct also supports `MonthlyBandwidthLimit`, `ZoneSecurityEnabled`, `OriginType` with no create flags |
| POST /pullzone/checkavailability | — | MISSING | No client method either. Name-availability preflight for create |
| GET /pullzone/loadFreeCertificate | `pull-zone hostname load-free-cert` | partial | `hostname` covered; `useOnlyHttp01` MISSING (client doesn't send it) |
| GET /pullzone/{id} | `pull-zone get` | partial | `includeCertificate` MISSING (also absent in client) |
| POST /pullzone/{id} | `pull-zone update` | partial | 122/143 body props have flags. 21 props unreachable — see section 2 |
| DELETE /pullzone/{id} | `pull-zone delete` | covered | |
| POST /pullzone/{id}/addAllowedReferrer | `pull-zone referrer allow` | covered | `Hostname` → `--value` |
| POST /pullzone/{id}/addBlockedIp | `pull-zone ip block` | covered | `BlockedIp` → `--value` |
| POST /pullzone/{id}/addBlockedReferrer | `pull-zone referrer block` | covered | |
| POST /pullzone/{id}/addCertificate | `pull-zone hostname add-cert` | covered | `Hostname`/`Certificate`/`CertificateKey` → `--hostname`/`--certificate`/`--key` |
| POST /pullzone/{id}/addHostname | `pull-zone hostname add` | covered | |
| POST /pullzone/{id}/purgeCache | `pull-zone purge` | covered | Spec body is `<unknown>`; CLI sends optional `CacheTag` via `--cache-tag`, empty body purges all |
| POST /pullzone/{id}/removeAllowedReferrer | `pull-zone referrer remove-allowed` | covered | |
| POST /pullzone/{id}/removeBlockedIp | `pull-zone ip unblock` | covered | |
| POST /pullzone/{id}/removeBlockedReferrer | `pull-zone referrer remove-blocked` | covered | |
| DELETE /pullzone/{id}/removeCertificate | `pull-zone hostname remove-cert` | covered | |
| DELETE /pullzone/{id}/removeHostname | `pull-zone hostname remove` | covered | |
| POST /pullzone/{id}/resetSecurityKey | — | MISSING | No client method. No way to rotate the token-auth key even though `--zone-security-enabled` exists |
| POST /pullzone/{id}/setForceSSL | `pull-zone hostname force-ssl` | covered | `ForceSSL` → `--enabled` |
| POST /pullzone/{id}/updatePrivateKeyType | — | MISSING | No client method. Hostname key type (RSA/EC) not switchable |
| POST /pullzone/{pullZoneId}/edgerules/addOrUpdate | `pull-zone edge-rule add` / `edge-rule update` | partial | Missing body props — see section 2 |
| DELETE /pullzone/{pullZoneId}/edgerules/{edgeRuleId} | `pull-zone edge-rule delete` | covered | `--rule-id` = GUID |
| POST /pullzone/{pullZoneId}/edgerules/{edgeRuleId}/setEdgeRuleEnabled | `pull-zone edge-rule enable` | covered | Client sends body `{"Id": pull_zone_id, "Value": enabled}` (client.rs:373) — matches spec shape |
| GET /pullzone/{pullZoneId}/optimizer/statistics | `pull-zone statistics --type optimizer` | covered | `dateFrom`/`dateTo`/`hourly` all flagged |
| GET /pullzone/{pullZoneId}/originshield/queuestatistics | `pull-zone statistics --type origin-shield` | covered | all query params flagged |
| GET /pullzone/{pullZoneId}/safehop/statistics | `pull-zone statistics --type safehop` | covered | all query params flagged |

### Core misc (14 ops)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /apikey | — | MISSING | No API-key management surface at all |
| GET /billing | `auth check` | covered | Full payload via `--format json`; table shows balance/charges/auto-recharge/bandwidth subset. No dedicated `billing` command |
| GET /billing/affiliate | — | MISSING | |
| GET /billing/payment-request-invoice/{id}/pdf | — | MISSING | Binary PDF download |
| GET /billing/payment-requests | — | MISSING | |
| GET /billing/summary | — | MISSING | |
| GET /billing/summary/{billingRecordId}/pdf | — | MISSING | Binary PDF download |
| GET /country | — | MISSING | Useful reference data for `--blocked-countries` values |
| POST /purge | `purge` | partial | `url` covered; `async` and `exactPath` MISSING (client hardcodes neither, client.rs:185) |
| GET /region | — | MISSING | (containers has its own `/regions`; core `/region` unexposed) |
| GET /search | — | MISSING | Global cross-resource search |
| GET /statistics | `statistics` | partial | 4/13 query params — see section 2 |
| GET /user/audit/{date} | — | MISSING | Audit log, 7 query params |
| POST /user/closeaccount | — | MISSING | Destructive; arguably should stay unexposed |

## 2. Flag-level gaps per command

### `pull-zone update` (POST /pullzone/{id}) — 21 of 143 spec body props with NO flag

The flag surface (built up over iters 44–47, 65) covers geo zones, log forwarding, optimizer, security/TLS/AWS-signing, vary headers, caching/perma-cache, origin host/DNS, timeouts/retries, origin shield, routing/sticky sessions, firewall/rate limiting, and websockets completely. The remaining unreachable props, by feature area:

- **Error pages (5)**: `ErrorPageEnableCustomCode`, `ErrorPageCustomCode`, `ErrorPageEnableStatuspageWidget`, `ErrorPageStatuspageCode`, `ErrorPageWhitelabel`
- **Preloading screen (7)**: `PreloadingScreenEnabled`, `PreloadingScreenCode`, `PreloadingScreenLogoUrl`, `PreloadingScreenShowOnFirstVisit`, `PreloadingScreenTheme`, `PreloadingScreenCodeEnabled`, `PreloadingScreenDelay`
- **Edge/middleware scripting (3)**: `EdgeScriptId`, `MiddlewareScriptId`, `EdgeScriptExecutionPhase` — no way to attach an edge script to a pull zone despite `hoppy script` existing
- **Magic Containers origin (2)**: `MagicContainersAppId`, `MagicContainersEndpointId` — despite `hoppy container` existing
- **Logging formats (2)**: `LogFormat`, `LogForwardingFormat` (enable/hostname/port/token/protocol ARE covered)
- **Bunny AI (1)**: `BunnyAiImageBlueprints` (`EnableBunnyImageAi` IS covered via `--enable-bunny-image-ai`)
- **Origin typing (1)**: `OriginType` — exists in the client struct but the CLI never sets it; relies on the API inferring type from `OriginUrl` vs `StorageZoneId`

None of these 21 are in the `UpdatePullZone` client struct either (except `OriginType`), so closing them needs client + CLI work. Note: referrer/IP/hostname/edge-rule props that overlap with dedicated subcommands (`AllowedReferrers`, `BlockedIps`, etc.) are ALSO settable in bulk as update flags — double-covered, not gaps.

### `pull-zone create` (POST /pullzone)

Flags: `--name`, `--origin-url`, `--storage-zone-id`, `--zone-tier` only. All other ~140 body props require a follow-up `update`. Intentional, but `MonthlyBandwidthLimit`/`ZoneSecurityEnabled` are already in the `CreatePullZone` client struct and would be cheap to expose.

### `pull-zone edge-rule add`/`update` (edgerules/addOrUpdate body)

| Spec body prop | Flag | Status |
|---|---|---|
| Guid | `--rule-id` (update only) | covered |
| ActionType | `--action-type` | covered |
| ActionParameter1/2 | `--action-param1/2` | covered |
| ActionParameter3 | — | MISSING (not in client struct) |
| Triggers[].Type, PatternMatches | `--trigger type:p1,p2` | covered |
| Triggers[].PatternMatchingType | — | MISSING — hardcoded `MatchAny` (pull_zone.rs:991) |
| Triggers[].Parameter1 | — | MISSING — hardcoded `None` (pull_zone.rs:992); breaks trigger types that need it (e.g. request-header, query-string triggers) |
| ExtraActions | — | MISSING (not in client struct) |
| TriggerMatchingType | `--trigger-matching-type` | covered |
| Description | `--description` | covered |
| Enabled | — | MISSING on add/update (client struct has `enabled`, CLI never sets it; separate `edge-rule enable` needs an extra call) |
| OrderIndex | — | MISSING — rule ordering not controllable |
| ReadOnly | — | MISSING (arguably meaningless to send) |

### `purge` (POST /purge)

- `url` → `--url` covered.
- `async` → MISSING.
- `exactPath` → MISSING — semantically important: controls exact-URL vs prefix/wildcard purging.

### `statistics` (GET /statistics) — 4/13 query params

Covered: `dateFrom`, `dateTo`, `pullZone`, `hourly`.
MISSING (9): `serverZoneId`, `loadErrors`, `loadOriginResponseTimes`, `loadOriginTraffic`, `loadRequestsServed`, `loadBandwidthUsed`, `loadOriginShieldBandwidth`, `loadGeographicTrafficDistribution`, `loadUserBalanceHistory`. None are in the client method signature (client.rs:731).

### `pull-zone list` / `pull-zone get`

`includeCertificate` MISSING on both.

### `pull-zone hostname load-free-cert`

`useOnlyHttp01` MISSING.

### `auth check` (GET /billing)

No query params in spec; covered. Table view is a curated subset (balance, charges, auto-recharge, payment method, bandwidth); full payload only via `--format json`.

## 3. CLI-only surface

- `pull-zone update --cache-expiration-time` → sends `CacheExpirationTime`, which is NOT in the spec's update body inventory (real-API field the spec dump omits).
- `pull-zone update --enable-extended-logging` → sends `EnableExtendedLogging`, also not in the spec body inventory.
- `pull-zone update --enable-logging` → sends `EnableLogging` (this one IS in the spec).
- `pull-zone referrer list`, `pull-zone ip list`, `pull-zone edge-rule list` — no spec endpoints; implemented client-side by reading `GET /pullzone/{id}` and projecting fields. Sensible convenience commands.
- `pull-zone list --all` — client-side auto-pagination, no spec counterpart.
- `pull-zone statistics --type` — CLI multiplexer over three distinct spec endpoints (optimizer / origin-shield queue / safehop).
- No CLI command in this domain hits a URL absent from the spec (all client URLs verified against inventory).

## 4. Observations

1. **Tier-type naming split**: create uses `--zone-tier` (`premium`/`volume`), update uses `--pull-zone-tier-type` (`standard`/`volume`). Both serialise to the same wire field `Type` (=0/1) (types.rs:1061, types.rs:1403). Same setting, two flag names, two vocabularies for value 0.
2. **Statistics selector params dropped**: the 9 missing `load*`/`serverZoneId` params mean the CLI always fetches the default payload; no server-zone scoping, no error/balance-history/geo-distribution series opt-in.
3. **Billing/account surface is minimal by design**: only `GET /billing` is reachable (as `auth check`). Billing summary, payment requests, invoice PDFs, affiliate details, API-key listing, global search, region/country reference lists, and the user audit log are all absent — a whole "account/admin" command group is missing. The two PDF endpoints would need binary-download handling (streaming per project rules).
4. **`/user/closeaccount` unexposed** — reasonable safety posture for a destructive account-level op; if ever added it must sit behind the confirm-prompt machinery.
5. **Zone Security half-covered**: `--zone-security-enabled` and `--zone-security-include-hash-remote-ip` exist, but `resetSecurityKey` doesn't — the key can be turned on but never rotated from the CLI.
6. **Edge-rule trigger parser is lossy**: `type:pattern1,pattern2` syntax cannot express per-trigger `PatternMatchingType` or `Parameter1` (hardcoded MatchAny/None in pull_zone.rs:988-993), and patterns containing commas can't be escaped.
7. **`loadFreeCertificate` is a GET that mutates** (spec quirk, not a CLI bug); CLI wraps it correctly under `hostname load-free-cert`.
8. **No deprecated ops flagged** in either inventory; nothing in the CLI targets a removed endpoint.

## Summary counts

- Total ops in scope: **41** (27 pullzone + 14 misc)
- Covered: **19** (18 pullzone + 1 misc)
- Partial: **8** (6 pullzone: list, create, get, update, loadFreeCertificate, edgerules/addOrUpdate; 2 misc: purge, statistics)
- Missing: **14** (3 pullzone: checkavailability, resetSecurityKey, updatePrivateKeyType; 11 misc: apikey, billing/affiliate, billing/payment-requests, 2 billing PDFs, billing/summary, country, region, search, user/audit, user/closeaccount)

5 most impactful gaps:
1. `POST /purge` drops `exactPath` (and `async`) — single-URL purge semantics (exact vs wildcard) not controllable from `hoppy purge`.
2. `GET /statistics` drops 9 of 13 query params (`serverZoneId` + all `load*` selectors) — no scoping or opt-in data series.
3. `pull-zone update` cannot set 21 body props — notably the entire error-page (5) and preloading-screen (7) groups, plus `EdgeScriptId`/`MiddlewareScriptId` (can't wire `hoppy script` output to a pull zone) and `MagicContainers*` origins.
4. `POST /pullzone/{id}/resetSecurityKey` missing — token security key can be enabled but never rotated.
5. Account/admin misc block missing wholesale (11 of 14 misc ops): API keys, billing summary/invoices/payment requests, region/country reference data, global search, audit log.
