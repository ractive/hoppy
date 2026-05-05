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

## Related
- [[api/bunny-api-client-patterns]] — how patterns handle these quirks
- [[api/bunny-api-overview]] — API overview
- [[decision-log]] — decisions influenced by these quirks
