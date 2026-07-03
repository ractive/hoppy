---
title: "Gap report: Edge Storage + Storage Zones"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - storage
  - storage-zone
---

# Storage gap report

Domain: Edge Storage API (`storage.bunnycdn.com`) + Storage Zone management (`api.bunny.net /storagezone`).
Sources: spec inventories (`inventories/storage.txt`, `inventories/core-split/storagezone.txt`), clap help dumps (`help/storage.txt`, `help/storage-zone.txt`, `help-tree.txt`), CLI source (`crates/hoppy-cli/src/commands/storage.rs`, `storage_zone.rs`, `cli.rs`), API client (`crates/bunny-net-api/src/storage/client.rs`, `crates/bunny-net-api/src/core/client.rs` + `types.rs`).

## 1. Endpoint coverage

### Edge Storage API (storage.bunnycdn.com)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET `/{zone}/{path}/` (List Files) | `hoppy storage ls` | covered | Spec has no query params; CLI adds `--region` host selector (see §3) |
| GET `/{zone}/{path}/{fileName}` (Download File) | `hoppy storage download` | covered | AccessKey header sent by client. Downloads whole body into memory, not streamed (see §4) |
| PUT `/{zone}/{path}/{fileName}` (Upload File) | `hoppy storage upload` | **partial** | `Checksum` header supported by client (`upload_file(..., checksum: Option<&str>)`) but CLI always passes `None` — no `--checksum` flag |
| DELETE `/{zone}/{path}/{fileName}` (Delete File) | `hoppy storage rm` | covered | Spec-level covered. Directory-delete (trailing-slash URL) semantics not reachable — see §4 |

### Storage Zone management (api.bunny.net)

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET `/storagezone` (List) | `hoppy storage-zone list` | **partial** | `includeDeleted` supported by client (`list_storage_zones(.., include_deleted)`) but CLI hardcodes `None` — no `--include-deleted` flag |
| POST `/storagezone` (Add) | `hoppy storage-zone create` | **partial** | `StorageZoneType` body property missing in client (`CreateStorageZone`) and CLI |
| POST `/storagezone/checkavailability` | — | **missing** | No CLI command, no client method (grep for `checkavailability` in crates/: no hits) |
| POST `/storagezone/resetReadOnlyPassword?id=` | — | **missing** | No CLI command, no client method |
| GET `/storagezone/{id}` (Get) | `hoppy storage-zone get` | covered | Password/ReadOnlyPassword redacted by default; `--reveal` opts in |
| POST `/storagezone/{id}` (Update) | `hoppy storage-zone update` | **partial** | `ReplicationZones` body property: client builder exists (`UpdateStorageZone::replication_zones`, types.rs:2433) but CLI exposes no `--replication-zones` flag — builder is dead code from the CLI's perspective |
| DELETE `/storagezone/{id}` | `hoppy storage-zone delete` | **partial** | `deleteLinkedPullZones` query param missing at BOTH client (`delete_storage_zone(id)` takes no extra arg, client.rs:440) and CLI level |
| POST `/storagezone/{id}/resetPassword` | — | **missing** | No CLI command, no client method |
| GET `/storagezone/{id}/statistics` | `hoppy storage-zone statistics` | covered | Both query params exposed |

`help-tree.txt` was grepped for `password|availab|checksum` — no other command exposes these operations elsewhere in the tree.

## 2. Flag-level gaps per command

### `hoppy storage upload` (PUT /{zone}/{path}/{fileName})

| Spec param | CLI flag | Status |
|---|---|---|
| path `storageZoneName` | `--zone` | OK |
| path `path` + `fileName` | `--remote-path` (split by `split_remote_path`) | OK |
| body octet-stream | `--file` (streamed via `reqwest::Body::wrap_stream`) | OK |
| header `Checksum` (SHA-256 hex) | — | **MISSING** — client parameter exists, CLI passes `None` (storage.rs:101) |
| header `AccessKey` | implicit (env/Core-API fallback) | OK |

No stdin upload (`--file` is required); spec body is arbitrary octet-stream so piping is a reasonable but absent mode.

### `hoppy storage download` / `ls` / `rm`

All spec params (path segments + AccessKey header) are mapped via `--zone` / `--remote-path`; no query/body params exist in spec. No flag gaps at spec level.

### `--region` enum (all four `storage` subcommands)

Accepted values validated in `StorageClient::new` against `VALID_REGIONS` (storage/client.rs:25): `storage, uk, ny, la, sg, syd, br, jh, se`. Help text only advertises "e.g. storage, la, sg, syd" — full list not shown; an invalid value errors at runtime with the full list. Not a clap `value_enum`, so shell completion won't offer values.

### `hoppy storage-zone list` (GET /storagezone)

| Spec param | CLI flag | Status |
|---|---|---|
| `page` | `--page` | OK |
| `perPage` | `--per-page` | OK |
| `search` | `--search` | OK |
| `includeDeleted` | — | **MISSING** (client supports it; handler passes `None` at storage_zone.rs:74,108) |

CLI-only: `--all` auto-pagination (perPage=1000 loop).

### `hoppy storage-zone create` (POST /storagezone)

| Spec body property | CLI flag | Status |
|---|---|---|
| `Name`* | `--name` | OK |
| `Region`* | `--region` | OK — free-form string, no client/CLI validation or enum (help says "e.g. DE, NY, LA, SG, SYD"; UK/SE/BR/JH not advertised); bad values fail server-side |
| `ReplicationRegions` | `--replication-regions` (comma-separated / repeatable) | OK — values unvalidated |
| `ZoneTier` | `--zone-tier` (`standard`=0, `edge`=1) | OK — spec leaves type as `?`; CLI enum matches known API values |
| `StorageZoneType` | — | **MISSING** (also absent from `CreateStorageZone` in types.rs:2378) |

### `hoppy storage-zone update` (POST /storagezone/{id})

| Spec body property | CLI flag | Status |
|---|---|---|
| `ReplicationZones` | — | **MISSING** — no flag; `UpdateStorageZone::replication_zones` builder exists but is never called by the CLI, so replication regions cannot be expanded post-create |
| `OriginUrl` | `--origin-url` | OK |
| `Custom404FilePath` | `--custom-404-file-path` | OK |
| `Rewrite404To200` | `--rewrite-404-to-200 <true|false>` | OK |

CLI requires at least one flag (bails otherwise) — good UX guard.

### `hoppy storage-zone delete` (DELETE /storagezone/{id})

| Spec param | CLI flag | Status |
|---|---|---|
| path `id`* | `--id` | OK |
| query `deleteLinkedPullZones` | — | **MISSING** (client method has no parameter for it either) |

### `hoppy storage-zone statistics` (GET /storagezone/{id}/statistics)

| Spec param | CLI flag | Status |
|---|---|---|
| `dateFrom` | `--date-from` (normalised via `date::normalise_datetime_opt`) | OK |
| `dateTo` | `--date-to` | OK |

### `hoppy storage-zone get` (GET /storagezone/{id})

`--id` only; matches spec. No gaps.

## 3. CLI-only surface (no spec counterpart)

Checked against client URLs — all of these hit documented-but-unspecced real API behavior or are pure client-side conveniences; none fabricate endpoints:

- **`--region <REGION>` on all `hoppy storage` commands** — the Edge Storage spec lists only `https://storage.bunnycdn.com/`; the real API has per-region hosts. Client builds `https://{region}.bunnycdn.com` (storage/client.rs:83). Correct intent, but see §4 for a hostname-format concern.
- **`hoppy storage ls` directory listing** — matches the spec's `GET /{zone}/{path}/` List Files; listing URL always gets a trailing slash (`listing_url`, client.rs:221). Fully legitimate.
- **`--all` on `storage-zone list`** — client-side auto-pagination loop, not an API param.
- **Access-key resolution fallback** (storage.rs:214–255) — `BUNNY_STORAGE_KEY` env var, else fetches the zone via Core `GET /storagezone?search=` and uses its `Password` field. Both are real API surfaces; the composition is CLI-only.
- **`BUNNY_STORAGE_URL`** base-URL override (auth.rs:61) — test/staging escape hatch.
- **Create auto-follow-up GET** — `storage-zone create` immediately re-fetches the zone (`get_storage_zone`) because the create response returns literal `"string"` placeholders for Password/ReadOnlyPassword, and prints with forced reveal (storage_zone.rs:147–160). Sensible workaround for a real API quirk.
- **Redaction layer** — Password/ReadOnlyPassword redacted unless `--reveal`; `--record` fixture redaction. Client-side only.
- **Per-segment URL encoding** of zone/path/filename (`encode_path_segments`) — hardening beyond spec.

No CLI flag in this domain sends a request parameter that the API does not document.

## 4. Observations

1. **Regional hostname format may be wrong.** Client builds `https://{region}.bunnycdn.com` (e.g. `la.bunnycdn.com`). Bunny's Edge Storage docs list regional endpoints as `{region}.storage.bunnycdn.com` (e.g. `la.storage.bunnycdn.com`, `uk.storage.bunnycdn.com`); only the primary is `storage.bunnycdn.com`. The `region="storage"` default happens to produce the correct primary host, and all unit tests + recorded fixtures only exercise `storage.bunnycdn.com` — non-default regions appear untested against the live API. Worth a live check before trusting `--region la` etc.
2. **Directory-delete semantics unreachable.** The real API deletes a directory recursively when the DELETE URL ends with `/`. `split_remote_path` trims trailing slashes and `file_url` never re-adds one, so `hoppy storage rm --remote-path images/` targets `.../images` (a file URL). The client docstring on `delete_file` claims "(or directory and all its contents)" but the CLI cannot actually produce the directory form. The spec itself omits directory deletion entirely.
3. **Download buffers the whole file in memory.** `download_file` returns `Bytes` via `response.bytes()` (storage/client.rs:153) and the CLI writes it afterwards. This violates the project's own streaming guidance (CLAUDE.md: prefer `bytes_stream()` for arbitrarily large blobs). Upload correctly streams. Also no progress bar granularity on download (spinner only).
4. **Checksum header** is present in the spec and in the client signature but has no CLI flag; the CLI never computes or forwards a SHA-256. Easy win for upload integrity.
5. **Password handling.** The full `Password` (read-write) is used for all storage operations, including read-only ones (`ls`, `download`). `ReadOnlyPassword` is fetched/displayed but never used for auth; a user can only opt into it by manually exporting `BUNNY_STORAGE_KEY`. With both reset endpoints missing (resetPassword, resetReadOnlyPassword), there is no CLI path to rotate a leaked storage credential at all.
6. **Spec inconsistency:** the List Files spec entry omits the `AccessKey` header that the real API requires; the client correctly sends it anyway.
7. **`StorageZoneType` and `ZoneTier`** are typed `?` in the spec (schema gap). CLI's `--zone-tier` values `standard`(0)/`edge`(1) match known API values; `StorageZoneType` is unmapped everywhere.
8. **Region value enums:** `storage-zone create --region` / `--replication-regions` accept arbitrary strings (no client-side enum for DE/NY/LA/SG/SYD/UK/SE/BR/JH); errors surface only from the server. Edge Storage `--region` is validated but against a hand-maintained list that names Falkenstein `storage` rather than `de`.
9. **List Files has no recursion/pagination** in the real API; `ls` is single-level only — inherent API limitation, not a CLI gap (no `--recursive` sugar exists either).

## Summary counts

- Total spec operations: **13** (4 Edge Storage + 9 Storage Zone management)
- Covered: **5** (storage ls, storage download, storage rm, storage-zone get, storage-zone statistics)
- Partial: **5** (storage upload — no `--checksum`; storage-zone list — no `--include-deleted`; storage-zone create — no `StorageZoneType`; storage-zone update — no `--replication-zones`; storage-zone delete — no `--delete-linked-pull-zones`)
- Missing: **3** (POST /storagezone/checkavailability; POST /storagezone/{id}/resetPassword; POST /storagezone/resetReadOnlyPassword)

Five most impactful gaps:
1. **Password rotation endpoints entirely absent** (`resetPassword`, `resetReadOnlyPassword`) — no way to rotate a leaked storage credential from the CLI; security-relevant.
2. **`--delete-linked-pull-zones` missing on `storage-zone delete`** — the query param is absent from client and CLI, so zone deletion with linked pull zones can't be done in one step.
3. **`--replication-zones` missing on `storage-zone update`** — replication cannot be expanded after create even though the API and the client builder both support it (dead client code).
4. **`--checksum` missing on `storage upload`** — spec'd integrity header, already plumbed in the client, one flag away.
5. **Regional endpoint hostname format (`{region}.bunnycdn.com` vs documented `{region}.storage.bunnycdn.com`)** — potential functional breakage for every non-default `--region` value; only the default host is covered by tests/fixtures.
