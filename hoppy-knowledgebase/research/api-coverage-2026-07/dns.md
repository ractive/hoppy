---
title: "Gap report: DNS Zones"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - dns
---

# DNS gap report

Domain: DNS Zones (core platform `/dnszone`), 17 spec operations.
Sources: `inventories/core-split/dnszone.txt`, `help/dns.txt`, `help-tree.txt`,
`crates/hoppy-cli/src/commands/dns.rs`, `crates/bunny-net-api/src/core/client.rs` (lines 448-644),
`crates/bunny-net-api/src/core/types.rs` (DNS types at 2488-2553, 2815-2889, 3055-3266).

## 1. Endpoint coverage

| # | METHOD path | CLI command | Status | Notes |
|---|-------------|-------------|--------|-------|
| 1 | GET /dnszone | `hoppy dns zone list` | covered | `--page`, `--per-page`, `--search` all present; bonus `--all` client-side auto-pagination |
| 2 | POST /dnszone | `hoppy dns zone create` | partial | Only `--domain`. Spec body also accepts a `Records` array (bootstrap zone with records) — not exposed; `CreateDnsZone` in types.rs has only `domain` |
| 3 | POST /dnszone/checkavailability | — | **missing** | No CLI command, no client method (`grep checkavailability` over crates/ = 0 hits). Body: `Name: string` |
| 4 | POST /dnszone/records/scan | `hoppy dns zone scan start` | covered | `--id` (ZoneId) / `--domain` (Domain), mutually exclusive via ArgGroup — matches spec's either/or body |
| 5 | GET /dnszone/{id} | `hoppy dns zone get` | covered | `--id`. Also reused by `dns record list` and `dnssec status` |
| 6 | POST /dnszone/{id} | `hoppy dns zone update` | partial | 6 of 8 body props exposed; `LogAnonymizationType` and `CertificateKeyType` missing (also absent from `UpdateDnsZone` struct) |
| 7 | DELETE /dnszone/{id} | `hoppy dns zone delete` | covered | Confirmation prompt + `-y` |
| 8 | POST /dnszone/{id}/dnssec | `hoppy dns zone dnssec enable` | covered | Prints DS record details for registrar |
| 9 | DELETE /dnszone/{id}/dnssec | `hoppy dns zone dnssec disable` | covered | Warning + confirmation prompt about dangling DS records |
| 10 | GET /dnszone/{id}/export | `hoppy dns zone export` | covered | Raw BIND to stdout; empty response falls back to a `get_dns_zone` call to synthesize a `;; zone ... 0 records` header |
| 11 | GET /dnszone/{id}/statistics | `hoppy dns zone statistics` | covered | `--date-from` / `--date-to` = both spec query params. Table view shows only `TotalQueriesServed`; the 4 chart maps only via `--format json` |
| 12 | POST /dnszone/{zoneId}/certificate/issue | `hoppy dns zone issue-cert` | covered | Spec body is `<unknown>`; CLI/client send no body. CLI adds a delegation hint on the API's structureless 500 |
| 13 | POST /dnszone/{zoneId}/import | `hoppy dns zone import` | covered | `--file` or stdin; prints successful/failed/skipped counts |
| 14 | PUT /dnszone/{zoneId}/records | `hoppy dns record add` | partial | 9 of 20 body props exposed — see section 2 |
| 15 | GET /dnszone/{zoneId}/records/scan | `hoppy dns zone scan results` | covered | `--id`, or `--domain` resolved client-side via zone-list exact match |
| 16 | POST /dnszone/{zoneId}/records/{id} | `hoppy dns record update` | partial | 7 of 21 body props exposed; even narrower than `add` (no `--port`/`--flags`/`--tag`) — see section 2 |
| 17 | DELETE /dnszone/{zoneId}/records/{id} | `hoppy dns record delete` | covered | `--zone-id` + `--record-id`, confirmation prompt |

## 2. Flag-level gaps per command

### `dns zone list` (GET /dnszone)

- `page` → `--page`, `perPage` → `--per-page`, `search` → `--search`. **No gaps.**

### `dns zone create` (POST /dnszone)

- `Domain` → `--domain` (required, matches spec's only required field).
- `Records: array<...>` → **MISSING** (no flag, no repeated-record syntax, no JSON-file input). Cannot create a zone pre-populated with records in one call; workaround is create + N `record add` calls (or `zone import`).

### `dns zone update` (POST /dnszone/{id})

| Spec body prop | CLI flag | Status |
|---|---|---|
| CustomNameserversEnabled | `--custom-nameservers-enabled <true\|false>` | ok |
| Nameserver1 | `--nameserver1` | ok |
| Nameserver2 | `--nameserver2` | ok |
| SoaEmail | `--soa-email` | ok |
| LoggingEnabled | `--logging-enabled <true\|false>` | ok |
| LoggingIPAnonymizationEnabled | `--logging-ip-anonymization-enabled <true\|false>` | ok |
| LogAnonymizationType | — | **MISSING** (enum, e.g. one-digit/drop; not in `UpdateDnsZone` struct either) |
| CertificateKeyType | — | **MISSING** (not in client struct either) |

CLI adds a sensible "at least one update flag required" guard (dns.rs:326-336).

### `dns zone statistics` (GET /dnszone/{id}/statistics)

- `dateFrom` → `--date-from`, `dateTo` → `--date-to`. **No flag gaps.** (Output-level: charts JSON-only.)

### `dns zone scan start` (POST /dnszone/records/scan)

- `ZoneId` → `--id`, `Domain` → `--domain`. **No gaps.**

### `dns record add` (PUT /dnszone/{zoneId}/records)

| Spec body prop | CLI flag | Status |
|---|---|---|
| Type | `--type` | ok — parser accepts all 16 enum values (A, AAAA, CNAME, TXT, MX, Redirect, Flatten, PullZone, SRV, CAA, PTR, Script, NS, SVCB, HTTPS, TLSA), case-insensitive (types.rs:2530-2553). Full parity with the client enum; spec Type is unenumerated (`?`) |
| Value | `--value` | ok (CLI makes it required; spec marks nothing required — bunny effectively requires it for most types) |
| Name | `--name` | ok |
| Ttl | `--ttl` | ok |
| Priority | `--priority` | ok |
| Weight | `--weight` | ok |
| Port | `--port` | ok |
| Flags | `--flags` | ok (u8) |
| Tag | `--tag` | ok |
| Comment | `--comment` | ok |
| PullZoneId | — | **MISSING** — `--type PullZone` is accepted but there is no way to pass the pull-zone id (not in `AddDnsRecord` struct); help text itself steers users to a CNAME workaround |
| ScriptId | — | **MISSING** — same problem for `--type Script` |
| Accelerated | — | **MISSING** |
| MonitorType | — | **MISSING** (monitoring: None/Ping/Http) |
| GeolocationLatitude | — | **MISSING** |
| GeolocationLongitude | — | **MISSING** |
| LatencyZone | — | **MISSING** |
| SmartRoutingType | — | **MISSING** (None/Latency/Geolocation) — smart/geo routing records cannot be configured |
| Disabled | — | **MISSING** — cannot create a record disabled, and no toggle later (record list shows a Disabled column, read-only) |
| EnviromentalVariables (sic) | — | **MISSING** (array<{Name,Value}> for Script records) |
| AutoSslIssuance | — | **MISSING** |

9/20 spec props exposed (Type+Value+8 optionals); 11 missing. All 11 are also absent from the `AddDnsRecord` client struct (types.rs:3059-3079), so closing these gaps needs client + CLI work.

### `dns record update` (POST /dnszone/{zoneId}/records/{id})

| Spec body prop | CLI flag | Status |
|---|---|---|
| Id | `--record-id` (path + body via `UpdateDnsRecord::new`) | ok |
| Type | `--type` (required) | ok, but forced re-specification — see observation O3 |
| Value | `--value` (required) | ok, same caveat |
| Name | `--name` | ok |
| Ttl | `--ttl` | ok |
| Priority | `--priority` | ok |
| Weight | `--weight` | ok |
| Comment | `--comment` | ok |
| Port | — | **MISSING** (present on `add`!) — SRV port cannot be changed; API may zero it on update |
| Flags | — | **MISSING** (present on `add`!) — CAA flags not updatable |
| Tag | — | **MISSING** (present on `add`!) — CAA tag not updatable |
| PullZoneId, ScriptId, Accelerated, MonitorType, GeolocationLatitude, GeolocationLongitude, LatencyZone, SmartRoutingType, Disabled, EnviromentalVariables, AutoSslIssuance | — | **MISSING** (same 11 as `add`) |

7/21 spec props exposed; 14 missing. `UpdateDnsRecord` (types.rs:3149-3164) is narrower than `AddDnsRecord` — the Port/Flags/Tag asymmetry is a client-struct gap, not just a CLI one.

### Zero-gap commands

`zone get`, `zone delete`, `zone export`, `zone import` (`--file`/stdin is CLI ergonomics for the raw text body), `dnssec enable`, `dnssec disable`, `zone scan results`, `record delete` — path params only, all mapped.

## 3. CLI-only surface

All client URLs in `core/client.rs` (lines 448-644) map 1:1 onto inventory paths — no client method hits a URL outside the spec. CLI-only *commands/flags* are compositions of spec endpoints:

- `hoppy dns record list --id` — no dedicated list-records endpoint exists in the spec; implemented by reading `Records` out of GET /dnszone/{id} (dns.rs:718-745). Legitimate convenience.
- `hoppy dns zone dnssec status --id` — reads `DnsSecEnabled` from GET /dnszone/{id}; when enabled it additionally calls POST /dnszone/{id}/dnssec (documented as idempotent) to enrich output with the DS record (dns.rs:527-584). Note: a "status" command performing a POST is surprising, even if side-effect-free.
- `dns zone list --all` — client-side auto-pagination (perPage=1000 loop).
- `dns zone scan results --domain` — client-side domain→zone-id resolution via GET /dnszone?search= with exact-match guard (dns.rs:674-693).
- `dns zone export` empty-body fallback — extra GET /dnszone/{id} to print a comment header for 0-record zones.
- `dns zone import --file` / stdin — local-file ergonomics over the raw import body.

No orphan flags found (no CLI flag serializing to a property absent from the spec).

## 4. Observations

- **O1 — checkavailability is the only fully missing endpoint.** POST /dnszone/checkavailability (body `Name`) has no client method and no command. A `hoppy dns zone check --domain <d>` would complete endpoint parity.
- **O2 — record bodies are the big surface gap.** The spec's record body has 20+ properties; the CLI exposes the "classic DNS" subset only. Everything bunny-specific — smart routing (SmartRoutingType, LatencyZone, Geolocation*), monitoring (MonitorType), linked resources (PullZoneId, ScriptId, Accelerated, EnviromentalVariables), Disabled, AutoSslIssuance — is unreachable, even though `--type` happily accepts PullZone/Script/Flatten. Accepting a type whose required companion field can't be passed is a UX trap the help text already has to apologize for.
- **O3 — `record update` is a lossy full-replace.** `--type` and `--value` are mandatory, and unset optional fields (Port/Flags/Tag are not even flags) are omitted from the POST body — updating e.g. the TTL of a CAA/SRV record risks clobbering tag/flags/port server-side. A read-modify-write (GET zone, merge, POST) or at least Port/Flags/Tag parity with `add` would fix this.
- **O4 — zone update lacks the two enum-typed props** (LogAnonymizationType, CertificateKeyType). Low frequency, but LogAnonymizationType matters to anyone using LoggingEnabled + IP anonymization compliance settings.
- **O5 — recheckdns:** bunny's public docs describe POST /dnszone/{id}/recheckdns (re-verify nameserver delegation), but it is absent from this spec inventory *and* from client/CLI (grep over crates/ = 0 hits). No gap counted against the 17-op inventory; worth tracking if the spec is regenerated.
- **O6 — statistics table view is minimal** — only TotalQueriesServed; the 4 chart series (queries served / normal / smart / by-type) require `--format json`. Flag parity is complete though.
- **O7 — zone create can't seed records** (spec `Records` array) — minor since `zone import` covers the bulk-load use case via BIND files.
- **O8 — import reads the whole zone file into memory** (`read_to_string`, dns.rs:403-414) before POSTing. Zone files are small in practice; noting only because the project guideline prefers streaming bodies for arbitrarily-large payloads.

## Summary counts

- Total spec operations: **17**
- Covered: **12**
- Partial: **4** (POST /dnszone create — no Records array; POST /dnszone/{id} update — 2 props missing; PUT records add — 11 props missing; POST records/{id} update — 14 props missing)
- Missing: **1** (POST /dnszone/checkavailability)
- 5 most impactful gaps:
  1. POST /dnszone/checkavailability — entire endpoint absent (client + CLI).
  2. Smart-routing/monitoring record fields (SmartRoutingType, MonitorType, LatencyZone, GeolocationLatitude/Longitude) missing from `record add`/`update` — bunny's headline smart-DNS features are unusable from the CLI.
  3. `record update` lacks `--port`/`--flags`/`--tag` (which `add` has) and forces `--type`/`--value` re-specification — updating SRV/CAA records is lossy/impossible.
  4. Linked-record fields (PullZoneId, ScriptId, Accelerated, EnviromentalVariables, AutoSslIssuance) missing while `--type PullZone`/`Script` are accepted — types are selectable but non-functional.
  5. No `Disabled` flag on add/update — records can't be disabled/re-enabled via CLI despite the list view displaying the column; plus zone-level `LogAnonymizationType` missing on `zone update`.
