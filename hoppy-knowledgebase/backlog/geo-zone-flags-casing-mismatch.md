---
title: "`EnableGeoZone*` fields deserialize to false because of casing mismatch with API"
type: backlog
date: 2026-05-31
status: resolved
priority: high
origin: dogfooding-2026-05-31
tags:
  - serde
  - core-api
  - pull-zone
  - critical-correctness
---

# Hoppy reports geo replication as `false` even when the API enabled it

`hoppy pull-zone update --enable-geo-zone-us true` reaches the API,
applies successfully, and the API echoes back **all five** geo zones as
`true`:

```jsonc
// raw response from bunny.net
"EnableGeoZoneUS": true,
"EnableGeoZoneEU": true,
"EnableGeoZoneASIA": true,
"EnableGeoZoneSA": true,
"EnableGeoZoneAF": true,
```

But hoppy's deserialised struct uses serde's default PascalCase rename, so it
expects:

```text
EnableGeoZoneUs   EnableGeoZoneEu   EnableGeoZoneAsia   EnableGeoZoneSa   EnableGeoZoneAf
```

…none of which match. Combined with `#[serde(default)]` on every field, every
flag silently deserialises to `false`. The user sees:

```
EnableGeoZoneUs    | false
EnableGeoZoneEu    | false
EnableGeoZoneAsia  | false
EnableGeoZoneSa    | false
EnableGeoZoneAf    | false
```

…even though the geo replication just got enabled. This is the same shape of
bug that iter-47 fixed for `BlockedIps` vs `BlockedIPs`.

## Repro

```sh
PZ=5940331
hoppy --debug pull-zone update --id $PZ --enable-geo-zone-us true --format json \
  | grep -E 'EnableGeoZone'
# Sent:  "EnableGeoZoneUs": true
# Recv:  "EnableGeoZoneUS": true  (capital S)
# CLI:   EnableGeoZoneUs: false  ← wrong
```

## Affected fields

In `crates/bunny-net-api/src/core/types.rs` around L699–L709 and
L1136–L1144:

| Hoppy field           | Hoppy default rename | Actual API key            |
|-----------------------|----------------------|---------------------------|
| `enable_geo_zone_us`  | `EnableGeoZoneUs`    | `EnableGeoZoneUS`         |
| `enable_geo_zone_eu`  | `EnableGeoZoneEu`    | `EnableGeoZoneEU`         |
| `enable_geo_zone_asia`| `EnableGeoZoneAsia`  | `EnableGeoZoneASIA`       |
| `enable_geo_zone_sa`  | `EnableGeoZoneSa`    | `EnableGeoZoneSA`         |
| `enable_geo_zone_af`  | `EnableGeoZoneAf`    | `EnableGeoZoneAF`         |

The same fields exist on both the read struct (L699) and the update body
(L1136). Update requests with the hoppy-side spelling have so far worked
(API is forgiving on input) but reads always defaultsuck to `false`.

## Suggested fix

Add explicit `#[serde(rename = "EnableGeoZoneUS")]` etc. on each field, on
both the read struct and the update request body. Also add a wiremock test
that uses the all-caps response payload to lock the contract.

While in the same file, audit other "country-code-like" suffix fields for
similar all-caps APIs (DNS country codes? CORS region codes?).

## Related

- iter-47 already fixed [[../iterations/iteration-47-pull-zone-firewall-and-rate-limiting]]
  for `BlockedIps` → `BlockedIPs`; this is the same class of bug.
- [[json-output-casing-inconsistency]]
