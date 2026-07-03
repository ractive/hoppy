---
title: "Gap report: Edge Scripting"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - edge-scripting
  - script
---

# Edge Scripting gap report

Domain: Edge Scripting / Compute API (`/compute/script*`, 23 spec operations).
Sources: `$SCRATCH/inventories/edge-scripting.txt`, `$SCRATCH/help/script.txt` (verified against `help-tree.txt`),
`/Users/james/devel/hoppy/crates/hoppy-cli/src/commands/script.rs`,
`/Users/james/devel/hoppy/crates/bunny-net-api/src/compute/{client.rs,types.rs}`.

## 1. Endpoint coverage

| # | METHOD path | CLI command | Status | Notes |
|---|-------------|-------------|--------|-------|
| 1 | GET /compute/script | `hoppy script list` | partial | `page`/`perPage`/`search` exposed (+`--all` auto-pagination). Query params `type` (array filter), `includeLinkedPullzones`, `integrationId` not sent by client or CLI. |
| 2 | POST /compute/script | `hoppy script create` | partial | Name/Code/ScriptType/CreateLinkedPullZone/LinkedPullZoneName all flagged. `Integration` body property has a client type (`SourceCodeIntegration`) but CLI hardcodes `integration: None` (script.rs:463). |
| 3 | GET /compute/script/{id} | `hoppy script get` | covered | |
| 4 | POST /compute/script/{id} | `hoppy script update` | covered | Both body props (`Name`, `ScriptType`) flagged; requires at least one. Client also serialises `Id` in the body (not in spec body — harmless, see Observations). |
| 5 | DELETE /compute/script/{id} | `hoppy script delete` | covered | `deleteLinkedPullZones` → `--delete-linked-pull-zones`. Confirmation prompt + `-y`. |
| 6 | GET /compute/script/{id}/code | `hoppy script code get` | covered | |
| 7 | POST /compute/script/{id}/code | `hoppy script code update` | covered | `Code` via `--code` (inline) or `--file` (read from disk); one is required (runtime check). |
| 8 | POST /compute/script/{id}/deploymentKey/rotate | `hoppy script rotate-deployment-key` | covered | Confirmation prompt + `-y`. |
| 9 | POST /compute/script/{id}/publish | `hoppy script publish` | covered | `Note` → `--note`. (Spec inventory oddly lists `uuid*` as a path param here too; the path has no `{uuid}` segment — spec quirk shared with op 10.) |
| 10 | POST /compute/script/{id}/publish/{uuid} | — | missing | No client method and no CLI surface. This is the publish-a-specific-release / rollback path. `script publish` has no `--uuid` flag. |
| 11 | GET /compute/script/{id}/releases | `hoppy script release list` | covered | `page`/`perPage` + `--all`. |
| 12 | GET /compute/script/{id}/releases/active | `hoppy script release get-active` | covered | |
| 13 | GET /compute/script/{id}/secrets | `hoppy script secret list` | covered | Values never returned by API; row shows id/name/last-modified. |
| 14 | POST /compute/script/{id}/secrets | `hoppy script secret add` | covered | `Name` → `--name`, `Secret` → `--value`. CLI requires `--value` though spec marks `Secret` optional (stricter than spec; benign). |
| 15 | PUT /compute/script/{id}/secrets | `hoppy script secret upsert` | covered | CLI requires both `--name` and `--value` (spec marks both optional). Client handles 200-with-body vs 204-no-body. |
| 16 | POST /compute/script/{id}/secrets/{secretId} | `hoppy script secret update` | covered | `Secret` → `--value` (required by CLI, optional in spec). |
| 17 | DELETE /compute/script/{id}/secrets/{secretId} | `hoppy script secret delete` | covered | Confirmation prompt + `-y`. |
| 18 | GET /compute/script/{id}/statistics | `hoppy script statistics` | partial | `dateFrom`/`dateTo`/`hourly` exposed. Query param `loadLatest` not sent by client or CLI. |
| 19 | PUT /compute/script/{id}/variables | `hoppy script variable upsert` | covered | Name/Required/DefaultValue all flagged. Client handles 204 with a placeholder result. |
| 20 | POST /compute/script/{id}/variables/add | `hoppy script variable add` | covered | Name/Required/DefaultValue all flagged (`--required` is a presence flag = spec's required boolean; absent → false, which satisfies the required field). |
| 21 | GET /compute/script/{id}/variables/{variableId} | — | missing (client-only) | `ComputeClient::get_variable` exists (client.rs:338) but no CLI command calls it. `script variable` offers list/add/update/delete/upsert — no `get`. Verified against full help-tree. |
| 22 | POST /compute/script/{id}/variables/{variableId} | `hoppy script variable update` | covered | `Required` (`--required true|false`), `DefaultValue` (`--default-value`); at least one required. |
| 23 | DELETE /compute/script/{id}/variables/{variableId} | `hoppy script variable delete` | covered | Confirmation prompt + `-y`. |

## 2. Flag-level gaps per command

### `script list` (GET /compute/script)

| Spec param | CLI flag | Status |
|---|---|---|
| page | `--page` | covered |
| perPage | `--per-page` | covered (client defaults page=1, perPage=1000 when omitted) |
| search | `--search` | covered |
| type (array of ScriptType) | — | MISSING (no `--type` filter; client `list_scripts` never sends it) |
| includeLinkedPullzones | — | MISSING (response type `EdgeScript.linked_pull_zones` exists, so output would deserialize if requested) |
| integrationId | — | MISSING |

### `script create` (POST /compute/script)

| Spec body prop | CLI flag | Status |
|---|---|---|
| Name | `--name` (required) | covered |
| Code | `--code` | covered (inline only — no `--file` here, unlike `code update`; minor UX gap) |
| ScriptType enum [0,1,2] | `--script-type dns|cdn|middleware` (required) | covered; enum fully mapped (dns=0, cdn=1, middleware=2) |
| CreateLinkedPullZone | `--create-linked-pull-zone` | covered |
| LinkedPullZoneName | `--linked-pull-zone-name` | covered |
| Integration (SourceCodeIntegration: repository, deploy config, integrationId) | — | MISSING — hardcoded `None` in handle_create; client/type support exists |

### `script update` (POST /compute/script/{id})

Name → `--name`, ScriptType → `--script-type`. No gaps. CLI enforces at-least-one-flag.

### `script delete` (DELETE /compute/script/{id})

deleteLinkedPullZones → `--delete-linked-pull-zones`. No gaps.

### `script code get` / `script code update`

No spec params beyond `Code`; `--code`/`--file` cover it. No gaps.

### `script publish` (POST /compute/script/{id}/publish[/{uuid}])

| Spec | CLI flag | Status |
|---|---|---|
| Note | `--note` | covered |
| uuid path variant (publish a specific/previous release) | — | MISSING — no `--uuid`, no client method (rollback flow impossible via CLI) |

### `script release list` / `release get-active`

page/perPage → `--page`/`--per-page` (+`--all`). No gaps.

### `script statistics` (GET /compute/script/{id}/statistics)

| Spec param | CLI flag | Status |
|---|---|---|
| dateFrom | `--date-from` | covered (normalised via crate::date) |
| dateTo | `--date-to` | covered |
| hourly | `--hourly` | covered |
| loadLatest | — | MISSING |

### `script variable *`

- `add`: Name/Required/DefaultValue → `--name`/`--required`/`--default-value`. No gaps. (`--required` is presence-only, so an explicit `Required=false` is the flag-absent default — semantically complete.)
- `update`: Required/DefaultValue → `--required true|false`/`--default-value`. No gaps.
- `upsert`: Name/Required/DefaultValue → all flagged. No gaps.
- `delete`: path params only. No gaps.
- `get` (single variable by ID): MISSING entirely at CLI level (see op 21).

### `script secret *`

- `add`: Name → `--name`, Secret → `--value`. CLI requires `--value`; spec says only `Name` is required. Stricter-than-spec, low impact.
- `update`: Secret → `--value` (CLI-required, spec-optional). Stricter-than-spec.
- `upsert`: Name/Secret → `--name`/`--value`, both CLI-required (spec: both optional). Stricter-than-spec.
- `list`/`delete`: no gaps.

## 3. CLI-only surface

Checked every URL the client builds — all map to spec paths; no phantom endpoints. CLI-only conveniences:

- `script variable list` — no spec endpoint lists variables; implemented by calling GET /compute/script/{id} and rendering the embedded `EdgeScriptVariables` array (script.rs:711-722). Legitimate composition, not an off-spec call.
- `--all` on `script list` and `script release list` — client-side auto-pagination loop (perPage=1000), not a spec param.
- `script code update --file` — reads the `Code` body property from a file; convenience over the spec's inline string.
- Global harness flags (`--format`, `--debug`, `--quiet`, `-y`, `--reveal`, `--reveal-env`, `--no-hints`, `--record`, `--no-redact`) — cross-cutting CLI machinery, no spec counterpart expected.

## 4. Observations

- **Rollback flow is the real hole.** Draft → `code update` → `publish` → `release list`/`get-active` all work, but `POST /publish/{uuid}` (re-activating an archived release) has no client method or CLI command. A `hoppy script publish --uuid <UUID>` (or `script release publish/rollback`) would complete the lifecycle; `release list` already surfaces the UUIDs needed.
- **Code get/set are cleanly covered** (`script code get`/`update`), including file input; `code get` table view truncates to 80 chars — full code available via `--format json`.
- **Statistics endpoint is covered** (some domains lack this); only `loadLatest` is unexposed. `EdgeScriptStatistics` type deserialises totals + cost fields.
- **`UpdateEdgeScript` serialises an `Id` field in the POST body** (types.rs:110-116) that the spec body doesn't define (only Name/ScriptType). Harmless (server ignores it) but off-spec.
- **Monthly request limits**: the spec create/update bodies define no request-limit property; `EdgeScript` responses carry `MonthlyRequestCount`/`MonthlyCost`/`MonthlyCpuTime`, which the CLI surfaces in `script get` detail output. Nothing to expose — no gap.
- **Pull-zone linking** is well covered on create (`--create-linked-pull-zone`, `--linked-pull-zone-name`) and delete (`--delete-linked-pull-zones`); the read side (`includeLinkedPullzones` on list) is the missing third leg, even though the response type already models `LinkedPullZones`.
- **Client hardcodes pagination defaults** (page=1, perPage=1000) instead of omitting the params — matches API defaults, cosmetic only.
- **Secret values are write-only** by API design; CLI list output correctly shows only id/name/last-modified.

## Summary counts

- Total spec operations: 23
- Covered: 18
- Partial: 3 (GET /compute/script — type/includeLinkedPullzones/integrationId filters; POST /compute/script — Integration body prop; GET .../statistics — loadLatest)
- Missing: 2 (POST /compute/script/{id}/publish/{uuid}; GET /compute/script/{id}/variables/{variableId} — client method exists but no CLI command)
- 5 most impactful gaps:
  1. `POST /compute/script/{id}/publish/{uuid}` — no rollback/re-publish of a previous release from the CLI.
  2. `script list` missing `--type`, `--integration-id`, `--include-linked-pullzones` filters (server-side filtering unusable).
  3. `script create` cannot attach a source-code `Integration` (GitHub deploy config) despite full client-type support.
  4. `script variable get` absent — single-variable fetch only reachable by listing via `script get`.
  5. `script statistics` missing `--load-latest`.
