---
title: Bunny.net Database (libSQL) API Research
date: 2026-05-07
tags:
  - bunny-net
  - api
  - database
  - libsql
status: research-complete
type: reference
---

# Bunny.net Database (libSQL) API Research

## Overview

bunny.net Database is a managed libSQL/SQLite service. There are two distinct
HTTP surfaces:

| Concern        | API                      | Base URL                                                |
|----------------|--------------------------|---------------------------------------------------------|
| Control plane  | bunny Database REST API  | `https://api.bunny.net/database`                        |
| Data plane     | libSQL HTTP protocol     | `https://<group_id>-<slug>.lite.bunnydb.net/v2/pipeline` |

The control plane is captured in `specs/database.json` (OpenAPI 3.1.0,
spec version `0.0.130`, fetched 2026-05-05 from
`https://api.bunny.net/database/docs/private/api.json`).

## Authentication

Two security schemes are listed in the spec:

- `Bearer` — bearer token (e.g. user JWT)
- `AccessKey` — header `AccessKey: $BUNNY_API_KEY`

hoppy uses `AccessKey` everywhere (no new auth wiring). The libSQL data
plane uses `Authorization: Bearer <jwt>` with a JWT minted via
`POST /v1/databases/{db_id}/auth/tokens` (or the v2 / group equivalents).

## Resource model

- **Database** (`db_<ulid>`) — a single libSQL database, lives inside a `DatabaseGroup`.
- **DatabaseGroup** (`group_<ulid>`) — a region cluster (storage region + primary regions + replica regions). Every DB is created inside a group.
- **Auth tokens** — JWTs scoped to either the whole group, a single DB, or a v2 DB.

### `Database` schema fields beyond the field-report bug report

- `block_reads`, `block_writes` — per-DB read/write block flags
- `allow_attach` — whether other databases may `ATTACH` this one
- `is_schema` — see schema-template feature below
- `schema` — parent DB whose schema this one inherits (`Option<String>`)
- `version` — libSQL version reported by the engine (e.g. `0.24.30`)
- `group_name` — display name of the parent group
- `current_size` / `size_max` — current bytes used / quota (string-encoded
  large integers in v1; v2 also exposes `current_size_bytes` / `size_max_bytes`
  as `uint64` and marks the v1 strings deprecated)

### Schema template feature

A Database with `is_schema=true` defines a schema other databases can
reference via the `schema` field. Documented here because the field bug
report didn't mention it — it's a real bunny feature exposed in the spec.

### Region taxonomy

Two parallel region concepts that look similar but are not interchangeable:

- `storage_region` — where the data is at rest. Flat strings like
  `eu-west-1`, `us-east-1`. Available list returned by `GET /v1/config`.
- `primary_regions[]` / `replicas_regions[]` — where the database is
  served from. Compute codes like `DE`, `FR`, `AMS`, `UK`, `NY`. The
  spec enum `PossibleRegion` lists 42 values as of `0.0.130`.

The CLI keeps these flag names distinct (`--storage-region` vs
`--primary-region` / `--replicas-region`) to avoid the easy confusion of
mixing them.

### Authorization scopes

`Authorization`, `Authorization2`, `Authorization3` in the spec are three
identical copies (`full-access` / `read-only`) under different names —
likely an artefact of the spec generator splitting per-tag. hoppy
collapses them into a single `Authorization` enum.

### Naming wart: invalidate vs revoke

- v1: `POST /v1/databases/{db_id}/auth/invalidate` (no body, 204)
- v2: `POST /v2/databases/{db_id}/auth/revoke` (no body, 204)

Same semantics, different verb. hoppy keeps the wire names per command
group (`db token invalidate` for v1, `db token revoke-v2` for v2) so
operators searching docs can find the matching endpoint.

## v1 vs v2 status

- v1 (`/v1/...`) — works, returns the wrapper-envelope shapes
  (`{"database": {...}}`).
- v2 (`/v2/...`) — exposes the same operations against a separate
  `Database2` type tree, plus per-DB `statistics` / `usage` /
  `active_usage`. **As of 2026-05-05, `POST /v2/databases` returns
  `{"error":"Internal error"}` (HTTP 500)**, while `POST /v1/databases`
  works. hoppy defaults `db create` to v1 and exposes v2 under `db v2`
  for forward compatibility.

## libSQL data plane

`Database.url` comes back as e.g. `libsql://group_01-my-app.lite.bunnydb.net/`.

- **Preserve casing AND trailing slash exactly** — libSQL clients reject
  normalised URLs.
- The HTTP equivalent for `SELECT 1` ping is
  `POST <https-host>/v2/pipeline` with body
  `{"requests":[{"type":"execute","stmt":{"sql":"SELECT 1"}},{"type":"close"}]}`
  and `Authorization: Bearer <jwt>`.

`hoppy db ping` does the URL-scheme rewrite (`libsql://` → `https://`)
internally and returns a typed `PingResult { ok, latency_ms, error }` so
it's parseable in CI gates.

## Slug-length footgun

The bunny API silently 500s on long slugs. The field report tested:
- `wa-admin-prod` (13 chars) — OK
- `wardrobe-assistants-admin` (25 chars) — `{"error":"Internal error"}`

hoppy validates slugs locally (`^[a-z][a-z0-9-]{0,23}$`) before the
API call. Conservative max of 24 chars; adjust `SLUG_MAX_LEN` in
`src/commands/database.rs` if upstream raises the limit.

## v2-only endpoints

- `GET /v2/databases/active_usage` — account-level active DB count + total size
- `GET /v2/databases/{db_id}/statistics` — chart series (rows R/W, storage, latency)
- `GET /v2/databases/{db_id}/usage` — aggregated usage in a window

These are not exposed in v1; hoppy's `db statistics`, `db usage`,
`db active-usage` route to the v2 endpoints regardless of `db v2`.

## Live metrics — non-standard request headers

`POST /v1/live/live_db` and `POST /v1/live/live_group` accept a JSON
body *and* a custom request header listing the IDs:

- `db-ids: id1,id2,...` for `live_db`
- `group-ids: id1,id2,...` for `live_group`

hoppy's client sets both the header and the JSON body so it works
regardless of which channel the bunny gateway prefers.

## JWT shape

JWTs come back from token-mint endpoints as opaque strings (~270 chars
typical). hoppy redacts them by default (`<set, length=N>`) and
prints raw bytes only when `--reveal` is set. This applies to JSON,
table, and text output identically.

## Forward-compat strategy

Following iter-19's pattern:

- Region enums (`PossibleRegion`, `RegionGroups`, `ChartUnit`) are
  modelled as `String` rather than typed enums — bunny adds region
  values silently. Loss of strong typing is the trade-off; the regions
  flow through unchanged.
- `LiveStatus` is a tagged union (`{"state": "Live" | "ReplicaOnly" | "Offline"}`)
  with a `#[serde(other)]` `Unknown` variant so a future state doesn't
  break deserialisation.
- `Authorization` only has two variants today; if bunny adds a third,
  a forward-compat wrapper similar to `serde_helpers::deserialize_repr_option`
  can be added to the input/output paths.

## Pagination

v1 uses bare-array responses (`{"databases": [...]}`, `{"groups": [...]}`).
v2 list adds a `DatabaseV2PageInfo { current_page, total_items, has_more_items }`
envelope — different shape from `bunny-api-core::PaginatedList`, so
hoppy keeps a parallel `ListDatabaseV2Response` type rather than
sharing the core pagination type.

## Open questions

- `optimal_single` was flagged in the field report as requiring a
  `cdn_server_token` header. The spec security on that endpoint is
  the same `Bearer | AccessKey` — verify with a live call.
- Whether `db statistics` / `db usage` are gated by tier on the
  account (the spec doesn't expose this).
