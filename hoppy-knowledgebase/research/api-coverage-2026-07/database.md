---
title: "Gap report: Bunny Database"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - database
---

# Database gap report

Domain: Bunny Database API (`https://api.bunny.net/database`), spec `0.0.130`, fetched 2026-05-05 from the *private* docs URL (`/database/docs/private/api.json`).
Sources checked: `$SCRATCH/inventories/database.txt` (34 ops), `$SCRATCH/help/db.txt` (full `hoppy db` clap tree, cross-checked against `help-tree.txt`), `/Users/james/devel/hoppy/crates/hoppy-cli/src/commands/database.rs`, `/Users/james/devel/hoppy/crates/bunny-net-api/src/database/{client.rs,types.rs}`, `/Users/james/devel/hoppy/specs/database.json`, research notes `hoppy-knowledgebase/api/bunny-database-research.md`.

## 1. Endpoint coverage

| # | METHOD path | CLI command | Status | Notes |
|---|---|---|---|---|
| 1 | GET /v1/config | `db config show` | covered | Renders storage/primary/replica regions |
| 2 | GET /v1/config/limits | `db config limits` | covered | |
| 3 | GET /v1/config/optimal | `db config optimal` | partial | Spec marks query `cdn_server_token` **required**; client (`get_optimal`) never sends it and CLI has no flag. Works live anyway (research note) |
| 4 | GET /v1/config/optimal_single | `db config optimal-single` (hidden) | missing (stubbed) | Clap subcommand exists but `#[command(hide = true)]` and handler unconditionally `bail!`s ("broken upstream — HTTP 400 missing field `cdn_server_token`"). Client method `get_optimal_single` exists but is never called; also omits `cdn_server_token` |
| 5 | GET /v1/databases | `db list` | covered | `group_id` → `--group-id` |
| 6 | POST /v1/databases | `db create` | covered | `slug`→`--slug`, `group`→`--group`; local slug regex validation (upstream 500 footgun) |
| 7 | GET /v1/databases/{db_id} | `db get --id` | covered | |
| 8 | DELETE /v1/databases/{db_id} | `db delete --id` | covered | Confirmation prompt + `-y` |
| 9 | POST /v1/databases/{db_id}/auth/invalidate | `db token invalidate --db-id` | covered | 204 handled via `check_status` |
| 10 | POST /v1/databases/{db_id}/auth/tokens | `db token mint` | covered | `authorization`→`--authorization` (default full-access), `expires_at`→`--expires-at`; JWT redacted unless `--reveal` |
| 11 | POST /v1/databases/{db_id}/fork (preview) | `db fork --id --target [--group]` | partial | **Spec drift**: spec body is `{slug*, date*}` (point-in-time fork); hoppy sends `{slug, group}` — required `date` never sent, `group` has no spec counterpart. See §2 |
| 12 | POST /v1/databases/{db_id}/list_versions (preview) | `db versions --id [--limit]` | partial | `limit`→`--limit`; `older_than`/`newer_than` hardcoded to `None` in handler — no flags |
| 13 | POST /v1/databases/{db_id}/restore (preview) | `db restore --id --version` | covered | `generation`→`--version`; destructive confirm |
| 14 | GET /v1/groups | `db group list` | covered | `search`→`--search` |
| 15 | POST /v1/groups | `db group create` | covered | All 4 body props flagged: `--display-name`, `--storage-region`, `--primary-region` (repeat), `--replicas-region` (repeat). Spec requires the region arrays; CLI lets them be empty (sends `[]`) |
| 16 | GET /v1/groups/{group_id} | `db group get --id` | covered | |
| 17 | DELETE /v1/groups/{group_id} | `db group delete --id` | covered | Confirmation prompt |
| 18 | PATCH /v1/groups/{group_id} | — | **missing** | No `db group update` command. Client `update_group` exists but is dead code from the CLI's perspective, and its payload only models `display_name` (spec also has `primary_regions`, `replicas_regions`) |
| 19 | GET /v1/groups/{group_id}/aggregated_usage | `db group usage --id --from --to` | covered | from/to normalised via `date::normalise_datetime` |
| 20 | POST /v1/groups/{group_id}/auth/generate | `db group generate-keys` | covered | `--authorization`, `--expires-at` |
| 21 | POST /v1/groups/{group_id}/auth/invalidate | `db group invalidate-keys --id` | covered | |
| 22 | GET /v1/groups/{group_id}/stats | `db group stats --id --from --to` | covered | |
| 23 | POST /v1/live/live_db | `db live --id ...` | covered | Body `db_ids` + non-standard `db-ids` request header (client sends both; documented quirk) |
| 24 | POST /v1/live/live_group | `db group live --id ...` | covered | Body `group_ids` + `group-ids` header |
| 25 | GET /v2/databases | `db v2 list` | covered | `page`→`--page`, `per_page`→`--per-page`, `search`→`--search`; plus CLI-only `--all` auto-pagination |
| 26 | POST /v2/databases | `db v2 create` | covered | All 4 body props flagged (`--name`, `--storage-region`, `--primary-region`, `--replicas-region`). Known 500 upstream as of 2026-05-05 — CLI prints a warning and points at v1 |
| 27 | GET /v2/databases/active_usage | `db active-usage` | covered | Promoted to top level (not under `db v2`) |
| 28 | GET /v2/databases/{db_id} | `db v2 get --id` | covered | |
| 29 | DELETE /v2/databases/{db_id} | `db v2 delete --id` | covered | Confirmation prompt |
| 30 | PATCH /v2/databases/{db_id} | — | **missing** | No `db v2 update` command. Client `update_database_v2` exists but unused, and its payload models a `name` field that is **not in the spec**, while the spec's `primary_regions`/`replicas_regions` are not modelled |
| 31 | PUT /v2/databases/{db_id}/auth/generate | `db token generate-v2 --db-id` | covered | Correct PUT verb in client |
| 32 | POST /v2/databases/{db_id}/auth/revoke | `db token revoke-v2 --db-id` | covered | |
| 33 | GET /v2/databases/{db_id}/statistics | `db statistics --id --from --to` | covered | Promoted to top level |
| 34 | GET /v2/databases/{db_id}/usage | `db usage --id --from --to` | covered | Promoted to top level |

## 2. Flag-level gaps per command

Only commands with a delta are listed; all others map every spec param 1:1.

### `db config optimal` (GET /v1/config/optimal)

- `cdn_server_token` (query, **required** in spec) → **MISSING** — no flag, never sent by `DatabaseClient::get_optimal` (client.rs:162-166). Endpoint appears to work without it (research open question suggests verifying live).

### `db config optimal-single` (GET /v1/config/optimal_single)

- Entire command is a hidden stub that `bail!`s (database.rs:848-854); `get_optimal_single` (client.rs:168-172) is never invoked.
- `cdn_server_token` (query, **required**) → **MISSING** in client — this is exactly why upstream returns HTTP 400. Adding the param would likely unbreak the endpoint and let the stub be un-hidden.

### `db fork` (POST /v1/databases/{db_id}/fork)

- `date` (body, **required** in spec, format date-time — the point-in-time to fork from) → **MISSING**. `ForkDatabasePayload` (types.rs:225-228) has no `date` field; no `--date`/`--at` flag. PITR-style forks are impossible from the CLI.
- `group` (body, sent by CLI via `--group`, defaults to the source DB's group) → **NOT IN SPEC** (`ForkDatabasePayload` in specs/database.json is exactly `{slug, date}`). Implementation was presumably validated against live behaviour, so this is spec/gateway drift — worth a live re-check, since the same spec version dates the research.

### `db versions` (POST /v1/databases/{db_id}/list_versions)

- `limit` → `--limit` (OK; u64 vs spec integer).
- `older_than` (body, string/date) → **MISSING** — hardcoded `None` (database.rs:429).
- `newer_than` (body, string/date) → **MISSING** — hardcoded `None` (database.rs:430). Without these there is no way to page/window the generation list beyond `--limit`.

### `db group update` — command absent (PATCH /v1/groups/{group_id})

- `display_name` → modelled in client payload but unreachable (no CLI command).
- `primary_regions` → **MISSING** in both client payload and CLI.
- `replicas_regions` → **MISSING** in both client payload and CLI.

### `db v2 update` — command absent (PATCH /v2/databases/{db_id})

- `primary_regions` → **MISSING** in both client payload and CLI.
- `replicas_regions` → **MISSING** in both client payload and CLI.
- Client payload's `name: Option<String>` (types.rs:338-341) has no spec counterpart — remove or verify live before exposing.

### `db group create` (POST /v1/groups) — minor

- Spec marks `primary_regions` and `replicas_regions` **required**; CLI flags are optional and default to empty vecs (sent as `[]`). Not a functional gap (field is always present in the JSON), just a looser contract than the spec.

## 3. CLI-only surface

- `db ping [--token-file]` — libSQL **data-plane** ping (`POST <db-host>/v2/pipeline` with `SELECT 1`), not part of the control-plane spec at all. Auto-mints a read-only token via op #10 when `--token-file` is absent. Intentional convenience; client.rs:532-594.
- `db fork --group` — body field `group` not present in spec's `ForkDatabasePayload` (see §2).
- `db v2 list --all` — client-side auto-pagination (loops `page` with `per_page=1000` until `has_more_items` is false). Pure CLI sugar over op #25.
- Live-metrics headers — client sends non-standard `db-ids`/`group-ids` request headers *in addition to* the spec JSON body (client.rs:495-520). Not in spec; documented in research notes as a gateway quirk.
- `UpdateDatabaseV2Payload.name` in the client — no spec counterpart (dead code today since no CLI command calls it).
- Everything else (`--format`, `--record`, `--reveal`, `--quiet`, `-y`, `--no-hints`, `--no-redact`, `--debug`) is hoppy-global plumbing, not API surface.

## 4. Observations

- **Pre-1.0, semi-private API.** Spec version `0.0.130` (0.x), served from `https://api.bunny.net/database/docs/private/api.json` — the whole service should be treated as beta; expect breaking drift between spec refreshes.
- **Three ops are explicitly "(preview)"** in spec summaries: fork (#11), list_versions (#12), restore (#13). These are the most likely to change shape (the fork `date` vs `group` drift in §2 may be exactly that).
- **Known upstream breakage baked into the CLI:** `POST /v2/databases` returns HTTP 500 as of 2026-05-05 (CLI warns and recommends v1); `GET /v1/config/optimal_single` returns HTTP 400 (subcommand hidden + stubbed). Both are dated markers — re-verify on the next spec refresh.
- **`cdn_server_token` inconsistency:** spec marks it a required query param on *both* optimal endpoints; hoppy sends it on neither; `/optimal` works, `/optimal_single` 400s. Suggests the spec is out of sync with the gateway in one direction or the other.
- **v1/v2 duplication:** the spec carries two full API generations. v2 drops the group concept (regions live on the database), renames invalidate→revoke, switches token mint to PUT, and adds pagination + statistics/usage. Hoppy hedges: v1 is the default surface, v2 is gated under `db v2` "(gated; some are broken upstream)". Long-term the v1/v2 split will need a migration decision once v2 stabilises.
- **Spec-generator artefacts** noted in research: `Authorization`/`Authorization2`/`Authorization3` are identical enums (hoppy collapses them); v1 sizes are string-encoded ints with v2 adding `*_bytes` u64 twins and deprecating the strings.
- Non-standard live-metrics ID headers (`db-ids`/`group-ids`) are undocumented in the spec; hoppy sends both header and body defensively.

## Summary counts

- Total spec operations: **34**
- Covered: **28**
- Partial: **3** (`GET /v1/config/optimal` — required `cdn_server_token` not sent; `POST .../fork` — required `date` missing, non-spec `group` sent; `POST .../list_versions` — `older_than`/`newer_than` not exposed)
- Missing: **3** (`PATCH /v1/groups/{group_id}` — no `db group update`; `PATCH /v2/databases/{db_id}` — no `db v2 update`; `GET /v1/config/optimal_single` — hidden stub that never calls the API)
- 5 most impactful gaps:
  1. No `db group update` (PATCH /v1/groups) — can't rename a group or change its primary/replica regions from the CLI; client payload also lacks the region fields.
  2. No `db v2 update` (PATCH /v2/databases) — v2 region changes impossible; the existing client payload models a non-spec `name` field instead of the spec's region arrays.
  3. `db fork` spec drift — required `date` (point-in-time fork) never sent, non-spec `group` sent instead; PITR forks unreachable and the payload may break on upstream spec enforcement.
  4. `db versions` lacks `--older-than`/`--newer-than` — generation history can't be windowed/paged beyond `--limit`.
  5. `cdn_server_token` unsupported on both optimal endpoints — leaves `optimal_single` permanently stubbed (HTTP 400) and `optimal` silently violating the spec's required-param contract.
