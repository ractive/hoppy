---
title: Iteration 20 — Bunny Database (libSQL) support
type: iteration
date: 2026-05-05
tags:
  - iteration
  - api-coverage
  - database
  - libsql
  - new-domain
status: in-progress
branch: iter-20/database
---

# Iteration 20 — Bunny Database (libSQL) support

**Goal:** Add bunny.net Database (libSQL/SQLite, branded "Bunny Database") to hoppy. Bunny exposes a clean REST control plane for database & group lifecycle plus auth-token minting; hoppy 0.1.0 has zero coverage today and operators are forced to provision via raw `curl`. After iter-20, `hoppy db` is the canonical CLI for the service.

## Context

- Field bug report: `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md` (2026-05-05) — surfaced during iter-9 admin go-live, captures the entire `curl` workaround that this iteration replaces.
- **OpenAPI spec captured at `specs/database.json`** (2026-05-05, version `0.0.130`, OpenAPI 3.1.0, fetched from `https://api.bunny.net/database/docs/private/api.json`).
- Control-plane base URL: `https://api.bunny.net/database` (confirmed in spec `servers[0].url` — same host as the core API; just a different path prefix, **not** a separate base URL).
- Auth: spec lists both `Bearer` and `AccessKey` security schemes — hoppy uses the existing `AccessKey: $BUNNY_API_KEY` header. No new auth wiring.
- Data plane: libSQL HTTP protocol at `https://<group_id>-<slug>.lite.bunnydb.net/v2/pipeline` with `Authorization: Bearer <jwt>`. Outside the control-plane crate; hoppy provides a `db ping` convenience using this endpoint.
- v1 vs v2: spec exposes both. As of 2026-05-05, `POST /v2/databases` returns `{"error":"Internal error"}` while `POST /v1/databases` works. **Default `db create` to v1**; expose v2 as `db create-v2` so users can opt in once upstream fixes the 500.
- The spec exposes 26 paths across 5 tag groups: `Config`, `Database`, `Database (v2)`, `DatabaseGroup`, `Live Metrics`. Iteration covers all five.

## Scope

### Spec capture & research

- [x] Save the OpenAPI doc to `specs/database.json` (matching the convention of `specs/core-platform.json` etc.) — done 2026-05-05
- [ ] Write `hoppy-knowledgebase/api/bunny-database-research.md` summarising:
  - control-plane base URL (`https://api.bunny.net/database`), auth (`AccessKey` or `Bearer`), response envelope (top-level `{ "database": {...} }` / `{ "group": {...} }` wrappers per the bug report — verify against spec)
  - resource model: `Database` / `DatabaseGroup` / auth tokens / config / live metrics
  - **`Database` schema fields not in the bug report**: `block_reads`, `block_writes`, `allow_attach`, `is_schema`, `schema` (parent-schema DB), `version` (libSQL version), `group_name`, `current_size`, `size_max` — surface in `db get`
  - **Schema-template feature**: a Database with `is_schema=true` defines a schema that other Databases can reference via the `schema` field. Document this — it's a real bunny feature the bug report didn't cover.
  - libSQL data plane URL pattern (`libsql://<group_id>-<slug>.lite.bunnydb.net/`)
  - region taxonomy: `storage_region` (flat: `eu-west-1`, `us-east-1`) vs `primary_regions[]` / `replicas_regions[]` (`PossibleRegion` enum: `AMS`, `DE`, `FR`, `UK`, …)
  - v1 vs v2 status, slug-length footgun, JWT shape
  - v2-only endpoints: `/v2/databases/active_usage`, per-database `/v2/databases/{db_id}/statistics`, per-database `/v2/databases/{db_id}/usage`
  - **Naming wart**: v2 uses `auth/revoke` where v1 uses `auth/invalidate` — document and decide on a unified CLI verb
  - Live Metrics: `/v1/live/live_db` and `/v1/live/live_group` accept POST + custom headers (`db-ids`, `group-ids`) — non-standard, document carefully

### `bunny-api-database` crate

- [ ] Scaffold `crates/bunny-api-database/` matching the shape of `bunny-api-stream` (Cargo.toml, src/lib.rs, src/client.rs, src/types.rs, tests/)
- [ ] Add to workspace `Cargo.toml` members
- [ ] Recording integration (`bunny-api-recording::capture_request` / `maybe_record_response`)
- [ ] Forward-compat enum strategy from iter-19 applied from day one to all repr-based enums
- [ ] No `.unwrap()` / `.expect()` outside tests; `anyhow::Context` with `?`

#### Types (`types.rs`)

Generated from the schemas in `specs/database.json` (component names in parens):

- [ ] `Database` (`Database`) — `id`, `name`, `url`, `group_id`, `group_name`, `block_reads`, `block_writes`, `allow_attach`, `is_schema`, `schema` (Option<String>), `version`, `size_max`, `current_size`. Note: spec also has a `Database2` for v2 responses — define separately.
- [ ] `DatabaseGroup` (`DatabaseGroup`) — `id`, `name`, `storage_region`, `primary_regions[]`, `replicas_regions[]`, status fields
- [ ] `AuthToken` — from `GenerateTokenDatabaseResponse` (`token`, `expires_at: Option<DateTime<Utc>>`)
- [ ] `TokenAuthorization` (`Authorization`) enum — exact serde strings from spec (likely `full-access` / `read-only`); apply iter-19 forward-compat fallback
- [ ] `Config` (`ListConfigAPIResponse`) — `storage_region_available[]`, `primary_regions[]`
- [ ] `Limits` (`LimitsResponse`) — surface in `db config limits`
- [ ] `Datapoint`, `LatencyChartData`, `LatencySingleRegionChart`, `ChartUnit` — for statistics responses
- [ ] `DBLiveStatus`, `GroupLiveStatus`, `LiveMetricsForDBPayload`, `LiveMetricsForDBResponse`, `LiveMetricsForGroupPayload`, `LiveMetricsForGroupResponse` — live metrics
- [ ] `PossibleRegion` enum — apply forward-compat fallback (bunny adds regions silently)
- [ ] `CountryCode` enum — same fallback
- [ ] `Generation`, `Database2`, `DatabaseV2PageInfo` — v2 response shapes
- [ ] `AppError` — used in 401/409/500 responses; map onto hoppy's existing error type
- [ ] Reuse `PaginatedList<T>` from `bunny-api-core` if `ListDatabaseResponse` shape matches; if not (likely uses `DatabaseV2PageInfo` for v2 pagination), document the deviation in `api/bunny-api-quirks.md`

#### Client methods (`client.rs`)

Mapped 1:1 to the 26 paths in `specs/database.json`.

**Config** (`Config` tag):
- [ ] `get_config()` → `GET /v1/config` — regions list
- [ ] `get_config_limits()` → `GET /v1/config/limits`
- [ ] `get_optimal()` → `GET /v1/config/optimal`
- [ ] `get_optimal_single()` → `GET /v1/config/optimal_single` (note: bug report flagged `cdn_server_token` requirement — verify against spec security)

**Database v1** (`Database` tag):
- [ ] `list_databases(params)` → `GET /v1/databases`
- [ ] `get_database(db_id)` → `GET /v1/databases/{db_id}`
- [ ] `create_database(body: CreateDatabasePayload)` → `POST /v1/databases` (`slug` + `group`)
- [ ] `delete_database(db_id)` → `DELETE /v1/databases/{db_id}`
- [ ] `fork_database(db_id, body: ForkDatabasePayload)` → `POST /v1/databases/{db_id}/fork`
- [ ] `restore_database(db_id, version)` → `POST /v1/databases/{db_id}/restore`
- [ ] `list_database_versions(db_id)` → `POST /v1/databases/{db_id}/list_versions`
- [ ] `mint_database_token(db_id, body: GenerateTokenDatabasePayload)` → `POST /v1/databases/{db_id}/auth/tokens`
- [ ] `invalidate_database_keys(db_id)` → `POST /v1/databases/{db_id}/auth/invalidate`

**Database v2** (`Database (v2)` tag):
- [ ] `list_databases_v2(params)` → `GET /v2/databases` *(broken upstream as of 2026-05-05 for create; verify list status)*
- [ ] `get_database_v2(db_id)` → `GET /v2/databases/{db_id}`
- [ ] `create_database_v2(body: CreateDatabaseV2Payload)` → `POST /v2/databases` *(returns 500 — implement but skip live tests; document)*
- [ ] `delete_database_v2(db_id)` → `DELETE /v2/databases/{db_id}`
- [ ] `get_active_usage_v2()` → `GET /v2/databases/active_usage`
- [ ] `get_database_statistics_v2(db_id, range)` → `GET /v2/databases/{db_id}/statistics`
- [ ] `get_database_usage_v2(db_id, range)` → `GET /v2/databases/{db_id}/usage`
- [ ] `mint_database_token_v2(db_id, body)` → `POST /v2/databases/{db_id}/auth/generate`
- [ ] `revoke_database_token_v2(db_id)` → `POST /v2/databases/{db_id}/auth/revoke`

**DatabaseGroup** (`DatabaseGroup` tag):
- [ ] `list_groups(params)` → `GET /v1/groups`
- [ ] `get_group(group_id)` → `GET /v1/groups/{group_id}`
- [ ] `create_group(body: CreateDatabaseGroupPayload)` → `POST /v1/groups`
- [ ] `delete_group(group_id)` → `DELETE /v1/groups/{group_id}`
- [ ] `get_group_stats(group_id)` → `GET /v1/groups/{group_id}/stats`
- [ ] `get_group_aggregated_usage(group_id)` → `GET /v1/groups/{group_id}/aggregated_usage`
- [ ] `generate_group_keys(group_id, body)` → `POST /v1/groups/{group_id}/auth/generate`
- [ ] `invalidate_group_keys(group_id)` → `POST /v1/groups/{group_id}/auth/invalidate`

**Live Metrics** (`Live Metrics` tag) — note: POST with custom request headers (`db-ids` / `group-ids`):
- [ ] `live_metrics_db(db_ids, payload)` → `POST /v1/live/live_db`
- [ ] `live_metrics_group(group_ids, payload)` → `POST /v1/live/live_group`

**Data-plane convenience** (not in spec; hoppy-only):
- [ ] `ping(database_url, token)` — POST to `<database_url>/v2/pipeline` with `{"requests":[{"type":"execute","stmt":{"sql":"SELECT 1"}},{"type":"close"}]}`. Takes `Database.url` so callers don't construct it.

#### Crate-level tests

- [ ] Wiremock + insta tests in `crates/bunny-api-database/tests/database_api.rs` for every method
- [ ] Capture fixtures in `fixtures/database/` (config, group_create, group_get, database_create, database_get, database_list, token_mint, …) — use `--record` flag once the client supports it

### CLI commands

Subcommand tree expanded to cover the full spec surface:

```text
hoppy db
  list                                          GET  /v1/databases
  get        --id                               GET  /v1/databases/{db_id}
  create     --slug --group                     POST /v1/databases
  delete     --id                               DELETE /v1/databases/{db_id}
  fork       --id --target                      POST /v1/databases/{db_id}/fork
  restore    --id --version                     POST /v1/databases/{db_id}/restore
  versions   --id                               POST /v1/databases/{db_id}/list_versions
  ping       --id [--token-file <path>]         libsql /v2/pipeline SELECT 1
  statistics --id [--from --to]                 GET  /v2/databases/{db_id}/statistics
  usage      --id [--from --to]                 GET  /v2/databases/{db_id}/usage
  active-usage                                  GET  /v2/databases/active_usage
  live       --id [--id ...]                    POST /v1/live/live_db (db-ids header)

hoppy db v2                                     (gated; v2 create currently broken)
  list                                          GET  /v2/databases
  get      --id                                 GET  /v2/databases/{db_id}
  create   --name --storage-region              POST /v2/databases  (broken upstream; warn loudly)
           --primary-region <r> [...]
           [--replicas-region <r> ...]
  delete   --id                                 DELETE /v2/databases/{db_id}

hoppy db group
  list                                          GET  /v1/groups
  get      --id                                 GET  /v1/groups/{group_id}
  create   --display-name --storage-region      POST /v1/groups
           --primary-region <r> [...]
           [--replicas-region <r> ...]
  delete   --id                                 DELETE /v1/groups/{group_id}
  stats    --id                                 GET  /v1/groups/{group_id}/stats
  usage    --id                                 GET  /v1/groups/{group_id}/aggregated_usage
  live     --id [--id ...]                      POST /v1/live/live_group (group-ids header)
  generate-keys   --id                          POST /v1/groups/{group_id}/auth/generate
  invalidate-keys --id                          POST /v1/groups/{group_id}/auth/invalidate

hoppy db token
  mint       --db-id                            POST /v1/databases/{db_id}/auth/tokens
             --authorization <full-access|read-only>
             [--expires-at <RFC3339>]
             [--reveal]                         (default: redact JWT in output)
  invalidate --db-id                            POST /v1/databases/{db_id}/auth/invalidate
  generate-v2 --db-id                           POST /v2/databases/{db_id}/auth/generate
  revoke-v2   --db-id                           POST /v2/databases/{db_id}/auth/revoke

hoppy db config
  show                                          GET  /v1/config
  limits                                        GET  /v1/config/limits
  optimal                                       GET  /v1/config/optimal
  optimal-single                                GET  /v1/config/optimal_single
```

CLI implementation tasks:
- [ ] New module `src/commands/database.rs`
- [ ] Wire into `src/cli.rs` as `Commands::Db { action: DbAction }` — use the short `db` (matches the bug report; shorter is better for daily ops)
- [ ] Subcommand groups: `DbAction { Database*, Group(GroupAction), Token(TokenAction), Config, Ping }`
- [ ] Repeatable region flags via `Vec<String>` (`--primary-region` repeatable, like edge-rule `--trigger`)
- [ ] Slug validator: `^[a-z][a-z0-9-]{0,N}$` — pick `N` empirically (long slugs return "Internal error" upstream; the report tested `wa-admin-prod` (13) OK, `wardrobe-assistants-admin` (25) failed). Conservative starting point: 24. Document the limit in `--help` and surface a clean error before hitting the API.
- [ ] Token redaction: by default `db token mint` prints `{ "length": 270, "authorization": "full-access", "expires_at": null }`. `--reveal` prints the raw JWT. Same default for `--format table`.
- [ ] `hoppy db ping --id <id>` flow: `get_database(id)` → mint a short-lived read-only token (or accept `--token-file`) → POST to `Database.url + "v2/pipeline"` with `SELECT 1`. Document the implicit token-mint side-effect; offer `--token-file` to skip.
- [ ] `long_help` on every flag mapping to a bunny enum (storage region, authorization, primary region) with the exact accepted values
- [ ] Examples in `--help` for `db create`, `db token mint`, `db ping`

### Testing

- [ ] Wiremock + insta snapshots for every CLI command (json + table where relevant)
- [ ] Slug validator unit tests (boundary length, invalid chars, leading digit)
- [ ] Token-redaction snapshot test — verify default output never contains the JWT
- [ ] `db ping` mock test stubs both control-plane (`get_database`) and data-plane (`/v2/pipeline`)
- [ ] Live E2E (`#[cfg(feature = "live-api")]`):
  - create group → create database → mint full-access token → ping → list versions → delete database → delete group
  - cleanup-stack pattern from `tests/support/mod.rs`
  - skip the v2 create until upstream 500 is fixed
- [ ] Auth/error tests: 401, 404 against control plane; libSQL `401` against data plane

### Documentation

- [ ] README features table: new row `Database (libSQL) | databases, groups, tokens, ping, config`
- [ ] `api/bunny-database-research.md` finalized with everything surfaced during implementation
- [ ] `api/bunny-api-quirks.md` updated with: v2 create returning 500, slug-length footgun (with the empirical limit found), `optimal_single` requiring `cdn_server_token`, libSQL URL casing/trailing-slash sensitivity
- [ ] Cross-reference in `api/bunny-api-overview.md`

## Implementation Notes

- **Spec version pinning.** `specs/database.json` is at API spec version `0.0.130` (captured 2026-05-05). Bunny is iterating fast — re-fetch and diff before each follow-up iteration. Track the version in the research note.
- **v1 vs v2 are separate type trees.** The spec defines `Database` and `Database2`, plus parallel payload/response schemas. Don't try to unify them in `bunny-api-database` — keep parallel modules `v1` and `v2`. CLI's `hoppy db v2 ...` subtree maps to the v2 module.
- **`Authorization` enum has three variants in the spec** (`Authorization`, `Authorization2`, `Authorization3`). Investigate during research — these may correspond to v1/v2/group-key authorization shapes. Map to one Rust type per scope; document in the research note.
- **Live metrics use custom request headers** (`db-ids`, `group-ids`) rather than path/query params. Wire the client method signature accordingly (`Vec<String>` of IDs joined into one header value). Add a wiremock test asserting the header is set.
- **libSQL URL preservation.** `Database.url` comes back as `libsql://<group_id>-<slug>.lite.bunnydb.net/` — preserve casing AND trailing slash exactly. libSQL clients reject normalised URLs. Don't pass through any URL parser that re-encodes.
- **Token printing.** Operators pipe `hoppy` output to logs constantly. Default to redaction; `--reveal` is the explicit opt-in. Apply to JSON and table outputs. The same logic should apply if we ever surface group keys.
- **v1 vs v2.** Default `db create` to v1. Expose v2 as a separate subcommand `db create-v2` so users who want it can opt in once upstream fixes the 500. Don't try to auto-fall-back on 500 — silent failure modes are worse than a clear error.
- **Region taxonomy is unusual.** `storage_region` is a flat region (`eu-west-1`); `primary_regions` and `replicas_regions` use compute codes (`DE`, `AT`, …). Don't conflate them in the CLI — keep the flag names distinct (`--storage-region` vs `--primary-region`).
- **Forward-compat enums.** Apply iter-19's pattern from day one. Bunny adds enum values silently — every typed enum needs a fallback variant.
- **Result types.** `db ping` should return a typed `PingResult { ok: bool, latency_ms: u64, error: Option<String> }` so it's machine-parseable in CI gates (the report's primary use case).

## Suggested test cases (from the bug report)

1. `hoppy db create --slug foo --group <id>` — succeeds, prints id + url + group.
2. `hoppy db ping --id <id>` against a freshly-created DB — returns OK within 30s.
3. `hoppy db token mint --db-id <id> --authorization=full-access` — returns redacted summary by default; `--reveal` prints the JWT.
4. `hoppy db delete --id <id>` — succeeds; `hoppy db get --id <id>` returns 404.
5. `hoppy db create --slug <very-long-slug>` — local validation fails before the API call, friendly error.
6. Auth-token round-trip: mint → use against `/v2/pipeline` → invalidate group keys → token now rejected.

## Estimated Complexity

| Phase | Complexity |
|-------|------------|
| Spec capture (DONE) + research note | Small |
| `bunny-api-database` crate scaffold | Small |
| 26 endpoints across v1 + v2 + Live Metrics | Medium–Large |
| `db ping` (control + data plane orchestration) | Small |
| CLI subcommand tree (db / db v2 / db group / db token / db config) | Medium |
| Slug validation, token redaction | Small |
| Mock + live tests (skip live for v2 create) | Medium |
| Docs | Small |
| **Total** | **Large** |

## Dependencies

- iter-19 (forward-compat enum strategy) ideally lands first so the new crate adopts the pattern from day one — soft dependency, not blocking.
- iter-21 (cross-cutting redaction layer) — `db token mint` consumes the redaction layer shipped by iter-21. **Recommended sequence is iter-21 → iter-20**. Note: iter-21 actually shipped a **post-serialization JSON walker** (`src/redact.rs`: `RedactConfig`, `placeholder()`, `redact_env_in_json`, `is_secret_field_name`, `RedactConfig::reveal_field()`) rather than a `Redacted<String>` newtype. `db token mint` should call a sibling walker (or `placeholder()` directly on the JWT field) gated by `RedactConfig::reveal_field()`; the global `--reveal` flag is already wired through. The redacted summary in the spec (`{ "length": 270, "authorization": "full-access", "expires_at": null }`) should swap the JWT for `placeholder(jwt)` (i.e. `<set, length=270>`) for shape-consistency with iter-21's env redaction.

## Related

- Field report: `../wardrobe-assistants.ch/kb/hoppy-bug-report-database-cli.md`
- [[development-roadmap]]
- [[adding-a-feature]]
- [[api/bunny-api-overview]]
- [[api/bunny-api-client-patterns]]
- [[api/bunny-api-quirks]]
- [[decision-log]]
