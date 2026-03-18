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
