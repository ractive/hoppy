---
title: bunny.net API Quirks
type: reference
created: 2026-03-18
status: active
---

# bunny.net API Quirks

Undocumented behaviors discovered by testing against the live API.

## List endpoints return different shapes depending on pagination params

**Affected endpoints:** `GET /pullzone` (likely all list endpoints)

- **Without `page`/`perPage` params:** returns a bare JSON array `[{...}, ...]`
- **With `page`/`perPage` params:** returns a paginated envelope `{"Items": [...], "CurrentPage": N, "TotalItems": N, "HasMoreItems": bool}`

The OpenAPI spec (`specs/core-platform.json`) only documents the paginated envelope shape. The bare array variant is undocumented.

**Workaround:** Always send `page=1&perPage=1000` (the API maximum) as defaults. This ensures a consistent `PaginatedList<T>` response.

## Authentication error response

A `401 Unauthorized` response returns:
```json
{"Message": "Authorization has been denied for this request."}
```

This does **not** include `ErrorKey` or `StatusCode` fields that other API errors may have. Our `ApiError` type handles this via `#[serde(default)]` on those fields, and backfills the HTTP status code.

## PascalCase field naming

All API responses use PascalCase field names (`Id`, `Name`, `OriginUrl`). Handled via `#[serde(rename_all = "PascalCase")]`.

## Update uses POST, not PATCH

Pull zone updates use `POST /pullzone/{id}` rather than `PATCH`. This is documented in the OpenAPI spec.

## Storage zone list rejects Accept header

**Affected endpoint:** `GET /storagezone`

Sending `Accept: application/json` header returns `401 Unauthorized`. Removing the Accept header works correctly with the same AccessKey.

Other Core API endpoints (`/pullzone`, `/storagezone/{id}`) work fine with the Accept header.

**Workaround:** Don't send `Accept: application/json` on storage zone list requests. Our `CoreClient` omits it by default (reqwest doesn't add it automatically).

## Storage API uses different auth header

**Affected endpoints:** All Storage API endpoints (`{region}.storage.bunnycdn.com`)

The Storage API uses `AccessKey` header (same as Core API) but requires a **per-zone storage password**, not the account API key. The password is available:
1. Via `BUNNY_STORAGE_KEY` environment variable
2. From the `Password` field in the storage zone details (`GET /storagezone/{id}`)

**Workaround:** Our CLI checks `BUNNY_STORAGE_KEY` first, then falls back to fetching the zone via Core API to extract the password.

## Storage zone PullZones field is nullable

**Affected endpoint:** `GET /storagezone` (list), `GET /storagezone/{id}` (get)

The `PullZones` field in storage zone responses can be `null` (not just an empty array). Our type uses `Option<serde_json::Value>` to handle this without failing deserialization.

## DNS record creation uses PUT

**Affected endpoint:** `PUT /dnszone/{zoneId}/records`

Record creation uses `PUT` instead of `POST`, which is the only endpoint in the entire API that does this. All other create operations use `POST`.

**Workaround:** `CoreClient::add_dns_record` uses `self.http.put()` explicitly.

## DNS records embedded in zone response

**Affected endpoint:** `GET /dnszone/{id}`

There is no standalone "list records" endpoint. Records are returned as a `Records` array inside the zone object when fetching a specific zone (`GET /dnszone/{id}`).

**Workaround:** `dns record list --zone-id <id>` fetches the full zone via `get_dns_zone()` and extracts the `records` field.

## DNS zone list has same pagination quirk

**Affected endpoint:** `GET /dnszone`

Same behavior as pull zones — returns bare array without pagination params, paginated envelope with them.

**Workaround:** Same as pull zones — always send `page=1&perPage=1000` defaults.

## EnviromentalVariables field is misspelled in API

**Affected:** DNS record model field `EnviromentalVariables` (missing "n" — should be "Environmental")

This is the actual field name in the bunny.net API. If we ever need to serialize/deserialize this field, we must use the misspelled name.

## Stream API uses PascalCase despite being a separate service

**Affected endpoints:** All Stream API endpoints (`video.bunnycdn.com`)

The Stream API OpenAPI spec describes fields in camelCase (`guid`, `title`, `videoLibraryId`), but the actual API responses use PascalCase (`Guid`, `Title`, `VideoLibraryId`) — same as the Core API. Our types use `#[serde(rename_all = "PascalCase")]` which works correctly against the live API.

## Stream API pagination uses different field names

**Affected endpoints:** All Stream API list endpoints

- Core API pagination: `Items`, `CurrentPage`, `TotalItems`, `HasMoreItems` with `perPage` query param
- Stream API pagination: `Items`, `CurrentPage`, `TotalItems`, `ItemsPerPage` with `itemsPerPage` query param

The response field `ItemsPerPage` replaces `HasMoreItems`. We compute `has_more_items` from `current_page * items_per_page < total_items`.

## Stream API uses per-library API key

**Affected endpoints:** All Stream API video/collection endpoints

The Stream API requires a per-library API key (not the account API key). Available from the `ApiKey` field in the VideoLibrary response (Core API `GET /videolibrary/{id}`).

**Workaround:** Check `BUNNY_STREAM_KEY` env var first, then fall back to fetching the library via Core API to extract `ApiKey`. Same pattern as Storage zones.

## Video upload is two-step with raw binary PUT

**Affected endpoint:** Video upload

1. `POST /library/{libraryId}/videos` — creates a video placeholder, returns GUID
2. `PUT /library/{libraryId}/videos/{videoId}` — uploads raw bytes with `Content-Type: application/octet-stream`

Re-uploading to an existing video ID returns 400. Must delete and recreate.

## Video library update uses POST

**Affected endpoint:** `POST /videolibrary/{id}`

Same as all other bunny.net update endpoints — uses POST, not PATCH.

## Magic Containers: `app delete` does not cascade to auto-managed Pull Zones

**Affected endpoint:** `DELETE /apps/{appId}` (Magic Containers API)

When a Magic Container app has a CDN endpoint, the bunny.net Magic Containers
service automatically provisions a Pull Zone in the Core API and stores its
id on the endpoint as `endpoint.pullZoneId`. Deleting the app does **not**
cascade to those auto-managed Pull Zones — they remain live and billable.

**hoppy mitigation:** `hoppy container app delete` enumerates endpoints and
refuses by default if any auto-managed PZ is detected. Operators choose:
- `--cascade` — delete the app, then DELETE each `pullZoneId` via the Core API.
- `--no-cascade` — delete the app and print the orphan IDs with a manual
  cleanup recipe (`hoppy pull-zone delete --id <id> --yes`).

**Discovery:** `endpoint.pullZoneId` is a stringified integer; "0" or empty
means "no auto-PZ" (Anycast / public-IP endpoints). Hoppy treats both as no
auto-PZ and skips them.

## Magic Containers: `template env` PUT replaces the whole array

**Affected endpoint:** `PUT /apps/{appId}/containers/{containerId}/env`

The endpoint accepts a flat `{key: value}` JSON body and **replaces** the
entire env-var set. There is no incremental endpoint. A bare PUT with `{}`
silently wipes all env vars and returns 200.

**hoppy mitigation:** `hoppy container template env` rejects bare invocations
and exposes:
- `--add KEY=VAL` / `--remove KEY` / `--update KEY=VAL` — read-modify-write
  granular merges (default mode).
- `--replace-all --env K=V …` — explicit destructive replace; if the new set
  shrinks the existing one, requires the operator to type "replace".
- `--clear` — explicit wipe; requires the operator to type "wipe".
- `--list` — names + redacted values; opt in with `--reveal` to see raws.

## Magic Containers: no logs-fetch endpoint

**Logs.** Bunny does not expose a logs-fetch endpoint for Magic Containers (verified against the LLM-friendly docs index at https://docs.bunny.net/llms.txt — 0 of 60 endpoints retrieve logs). The only path is **log-forwarding configuration** (5 endpoints): the operator points Bunny at a syslog receiver they control. `hoppy container logs` automates this round-trip with a transient receiver + tunnel.

## Magic Containers: env-var values are returned in plaintext

**Affected endpoints:** `GET /apps/{appId}` (`containerTemplates[*].environmentVariables`),
`GET /apps/{appId}/containers/{containerId}`, and the `PUT .../env` response.

The API returns every env-var value verbatim, including secrets like
`BETTER_AUTH_SECRET`, `DATABASE_AUTH_TOKEN`, `RESEND_API_KEY`. This is
unavoidable on the wire but ends up in `hoppy` output (and the operator's
terminal scrollback) by default.

**hoppy mitigation:** the cross-cutting redaction layer rewrites every
`environmentVariables[*].value` to `<set, length=N>` (or `<unset>` for empty
values) in JSON, table, and text output. Opt in with `--reveal` (all) or
`--reveal-env KEY` (a single var). The flag must be passed explicitly on each
invocation; it cannot be enabled via an environment variable.

## Repr-based enums grow new variants without API versioning

**Affected enums:** `OriginType`, `PullZoneType`, `EdgeRuleActionType`,
`TriggerType`, `MatchingType`, `DnsRecordType` and the entire shield/stream/
compute repr-enum surface.

bunny adds new integer values (e.g. `OriginType: 5 = MagicContainerEndpoint`,
which appears on every Magic-Container-backed Pull Zone) without bumping a
spec version. The naive `Deserialize_repr` impl fails with
`invalid value: 5, expected one of: 0, 2, 3, 4` and the whole response is
unusable.

**hoppy mitigation (iter-19):** every `Option<EnumType>` field on a response
struct uses [`bunny_api_core::serde_helpers::deserialize_repr_option`], which
deserialises unrecognised integers to `None` instead of erroring. Round-trips
lose the original integer (the field re-serialises as absent) — documented
trade-off, since the alternative is the entire response failing. `DnsRecord.record_type`
was changed from `DnsRecordType` to `Option<DnsRecordType>` for the same reason.

## Pull Zone origin: Storage-Zone-backed zones omit OriginUrl

`POST /pullzone` accepts either:
- `OriginUrl` (HTTP/HTTPS origin), or
- `StorageZoneId` (Storage-Zone-backed Pull Zone) — `OriginUrl` is omitted
  or sent as the empty string.

The OpenAPI spec marks `OriginUrl` as required, but bunny accepts the body
without it when `StorageZoneId` is set. `hoppy` enforces "exactly one" at the
CLI layer via a clap `ArgGroup` on `pull-zone create`.

## Storage Zone passwords are returned verbatim

`GET /storagezone/{id}` returns `Password` and `ReadOnlyPassword` in plaintext
— required for authenticating against the storage endpoint. Earlier iterations
hid them via `skip_serializing` in the type, but operators bootstrapping
storage credentials had to fall back to raw `curl`. `hoppy` (iter-19) now
serialises them and the CLI redacts them by default (`<set, length=N>`); pass
`--reveal` to surface raw values.

## DNS record types: Flatten / PullZone caveats

`dns record add --type Flatten` is accepted by clap but the API may reject it
with "Unknown record type" depending on the zone configuration —
`Flatten` is a smart-routing-only feature that requires DNS-routing-capable
zones. `--type PullZone` only accepts standard, non-managed Pull Zone IDs;
auto-managed Pull Zones backing Magic Containers fail with "pull zone ID is
not valid". For Magic-Container-backed CDN endpoints, use a `CNAME` to the
`b-cdn.net` hostname instead.

## Database (libSQL): v2 create returns 500

**Endpoint:** `POST /v2/databases`

As of spec version `0.0.130` (2026-05-05), `POST /v2/databases` returns
`{"error":"Internal error"}` (HTTP 500) for valid payloads, while
`POST /v1/databases` works. hoppy defaults `db create` to v1 and exposes
v2 only under `db v2 create` (with a stderr warning) so users can opt
in once upstream fixes the issue.

## Database (libSQL): slug-length footgun

**Endpoint:** `POST /v1/databases`

Long slugs cause an opaque `{"error":"Internal error"}` 500 — the API
does not validate slug length and it leaks through to the storage layer.
Empirically, 13 chars (`wa-admin-prod`) succeeds; 25 chars
(`wardrobe-assistants-admin`) fails. hoppy validates locally with
`^[a-z][a-z0-9-]{0,23}$` (max 24 chars) before any HTTP call.

## Database (libSQL): URL preservation

**Field:** `Database.url` — e.g. `libsql://group_01-my-app.lite.bunnydb.net/`

libSQL clients reject URLs with normalised casing or stripped trailing
slashes. Pass the value through unchanged — never via a URL parser
that re-encodes. `hoppy db ping` only rewrites the scheme
(`libsql://` → `https://`) and appends `/v2/pipeline`; everything
else is preserved byte-for-byte.

## Database (libSQL): live metrics use custom request headers

**Endpoints:** `POST /v1/live/live_db`, `POST /v1/live/live_group`

The IDs to query are passed both in the JSON body *and* in a
non-standard request header (`db-ids` or `group-ids`, comma-joined).
The spec only documents the body; the header is the load-bearing
input. hoppy's `DatabaseClient::live_metrics_db` /
`live_metrics_group` set both.

## Database (libSQL): v1 `auth/invalidate` vs v2 `auth/revoke`

Same semantics, different verb. v1 uses `auth/invalidate`; v2 uses
`auth/revoke`. Both return `204 No Content` and take no body. hoppy
keeps both verbs to make endpoint-to-doc lookups easy.

## Database (libSQL): `optimal_single` may require `cdn_server_token`

**Endpoint:** `GET /v1/config/optimal_single`

The field bug report flagged this as requiring a `cdn_server_token`
header. The spec security still lists only `Bearer | AccessKey`, so
this might be a server-side residual check. Open question — verify
with a live call when next testing.

## Optimizer fields

### `OptimizerClasses` — documented as string, returns array when empty

**Field:** `PullZoneModel.OptimizerClasses`

The bunny.net API documents `OptimizerClasses` as a JSON string
(a serialised map of class-name → URL parameters, e.g.
`"{\"thumb\":\"width=200,quality=80\"}"`). However, the live API
returns `[]` (an empty JSON array) when no classes are configured.

hoppy handles this with `deserialize_string_lossy_option`: any
non-string JSON value (including arrays and objects) deserialises to
`None`. When the field holds a string it is preserved as `Some(s)`.
This means an empty-array response is silently dropped on read.

### `OptimizerPricing` — server-set float, not writable

**Field:** `PullZoneModel.OptimizerPricing`

The spec hints at an integer type, but the live API returns a float
(e.g. `9.5`). hoppy maps it to `Option<f64>` on `PullZone` and
excludes it entirely from `UpdatePullZone` (server-set, ignored on
writes).

### `OptimizerWatermarkPosition` — repr-based enum, may grow

**Field:** `PullZoneModel.OptimizerWatermarkPosition`

Values: `0=TopLeft, 1=TopRight, 2=BottomLeft, 3=BottomRight, 4=Center`.
Bunny may add new positions in future API versions. hoppy uses
`deserialize_repr_option` so unknown future values deserialise to
`None` instead of panicking.

### `OptimizerMinifyCSS` — irregular capitalisation

**Field:** `PullZoneModel.OptimizerMinifyCSS`

PascalCase renaming of `optimizer_minify_css` would produce
`OptimizerMinifyCss`, but the wire format uses all-caps `CSS`.
hoppy applies `#[serde(rename = "OptimizerMinifyCSS")]` explicitly
on this field in both the response and request structs.

## Related
- [[api/bunny-api-client-patterns]] — how patterns handle these quirks
- [[api/bunny-api-overview]] — API overview
- [[api/bunny-database-research]] — full Database API research note
- [[decision-log]] — decisions influenced by these quirks
