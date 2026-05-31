---
title: "`pull-zone update` is missing 33 toggle/boolean flags from the API surface"
type: backlog
date: 2026-05-31
status: planned
priority: medium
origin: dogfooding-2026-05-31 (post-iter-42)
---

# `pull-zone update` exposes a fraction of the API's toggle surface

The bunny.net Pull Zone API returns ~45 boolean/toggle fields on every
zone (Enable*/Disable*/Allow*/Block*/Add*/Use*/Verify*/Ignore*). The CLI
covers **12** of them, leaving **33 confirmed coverage gaps** where the
user has to fall back to the dashboard.

Discovered by the dogfooding TLS attempt
(`hoppy pull-zone update --id … --enable-tls-1-1 false` → "unexpected
argument"). The systematic audit:

```sh
# raw API response (use --debug to bypass the CLI's curated subset)
hoppy --debug --format json pull-zone get --id <id> 2>&1 \
  | awk '/^<<< /{flag=1; sub(/^<<< /,""); print; next} flag && !/^[<>]/{print}' > raw.json

# field names that look like toggles
jq -r 'to_entries[]
       | select((.key | test("^(Enable|Disable|Allow|Block|Force|Use|Ignore|Add|Verify)"))
                and (.value | type | test("boolean|integer|number")))
       | .key' raw.json | sort -u > api.txt

# CLI flags converted to PascalCase
hoppy pull-zone update --help | grep -oE -- '--[a-z][a-z0-9-]+' \
  | awk -F- '{out=""; for(i=2;i<=NF;i++) out=out toupper(substr($i,1,1)) substr($i,2); print out}' \
  | sort -u > cli.txt

comm -23 api.txt cli.txt
```

## The 33 gaps, grouped by theme

**Security / compliance (highest priority — these directly affect
PCI/SOC2 posture):**

- `EnableTLS1`, `EnableTLS1_1` — toggle deprecated TLS versions.
- `EnableAutoSSL`, `DisableLetsEncrypt` — cert provisioning controls.
- `VerifyOriginSSL` — origin TLS chain verification.
- `EnableAccessControlOriginHeader` — CORS.

**Caching behaviour:**

- `EnableCacheSlice`, `EnableSmartCache`, `EnableSafeHop`
- `IgnoreQueryStrings`, `EnableQueryStringOrdering`
- `UseStaleWhileUpdating`, `UseStaleWhileOffline`, `UseBackgroundUpdate`

**Vary headers (cache-key partitioning):**

- `EnableWebPVary`, `EnableAvifVary`, `EnableCookieVary`,
  `EnableCountryCodeVary`, `EnableCountryStateCodeVary`,
  `EnableHostnameVary`, `EnableMobileVary`

**Origin / edge:**

- `AddHostHeader`, `AddCanonicalHeader`
- `EnableOriginShield`, `EnableRequestCoalescing`
- `EnableBunnyImageAi`

**Request blocking:**

- `BlockPostRequests`, `BlockRootPathAccess`, `BlockNoneReferrer`
- `DisableCookies`

**Logging / misc:**

- `EnableLogging`, `EnableExtendedLogging`
- `EnableWebSockets`

## Root cause: forgotten, not curated

The CLI's `--format json` output is **not** filtering these fields away.
Confirmed by diffing the raw API response against the `PullZone`
deserialization struct in `crates/bunny-net-api/src/core/types.rs:460`:

- API returns 164 fields, ~45 of them toggle-shaped.
- `PullZone` struct has 56 modeled fields.
- The 12 toggles the CLI exposes on `update` are exactly the ones the
  struct models.
- `comm -23 api_toggles struct_pascal_case` returns the same 33-field
  list as the CLI update gap.

Serde silently drops unknown fields at deserialization (no
`#[serde(deny_unknown_fields)]`), so anything the API sends that the
struct doesn't declare vanishes before reaching CLI output. The struct
comment at types.rs:455 confirms the design ("Fields that bunny.net may
omit on older zones are annotated with `#[serde(default)]`") — leniency
on missing fields, with no intent to hide anything from users.

This means **each gap is a two-layer fix, not a one-layer CLI tweak**:

1. Add the field to the `PullZone` struct (so `get` / `list` surface it).
2. Add the field to the `UpdatePullZone` payload (so `update` accepts it).
3. Add the CLI flag in `PullZoneAction::Update` (cli.rs).
4. Forward through in `crates/hoppy-cli/src/commands/pull_zone.rs`.
5. Help text matching the bunny.net dashboard label.
6. e2e snapshot refresh for the new `--help` shape.

## Suggested approach

Treat this as one iteration (not 33). The pattern is repetitive — a
code-gen pass from `specs/core-platform.json` into both the struct and
the CLI args could land them all at once, but a hand-written pass per
group is also fine. The **security** group is the only one that's
genuinely blocking compliance use cases — the others are
quality-of-life.

## Out of scope

- The same audit on storage-zone / dns zone / container app / stream
  library update commands. Worth a follow-up.
- Non-toggle fields (strings, numbers like `CacheControlMaxAgeOverride`)
  — most of those *are* exposed; this audit was specifically about the
  toggle surface.
- Two minor "false-positive" gaps in the diff: `EnableGeoZoneAF` /
  `ASIA` / `EU` / `SA` / `US` (ALLCAPS) appear alongside their
  camelCase twins in the API response; the CLI's `--enable-geo-zone-*`
  flags cover them — they were excluded from the 33 above.
