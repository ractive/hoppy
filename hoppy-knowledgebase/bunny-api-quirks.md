---
title: bunny.net API Quirks
type: reference
created: 2026-03-18
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
