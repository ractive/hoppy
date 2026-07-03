---
title: "Gap report: Magic Containers"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - containers
  - magic-containers
---

# Magic Containers gap report

Domain: Magic Containers API (`https://api.bunny.net/mc`)
Authoritative reference: fresh OpenAPI spec `$SCRATCH/fresh-specs/magic-containers.json`
(inventory: `$SCRATCH/inventories/magic-containers.txt`, 52 operations), cross-checked against
8 hand-written notes in `hoppy-knowledgebase/api/magic-containers/` (incl. `templates-registries/`).
CLI surface: `$SCRATCH/help/container.txt` (61 commands, 51 leaf commands).
Client: `crates/bunny-net-api/src/containers/client.rs` (41 endpoint methods).
CLI impl: `crates/hoppy-cli/src/commands/container.rs`.

## 1. Endpoint coverage

Status legend: **covered** = CLI command exists and exposes all documented inputs;
**partial** = CLI command exists but documented body/query properties are missing;
**missing** = no CLI command reaches the operation.

### Applications

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /apps | `container app list` (alias: `container list`) | covered | `--cursor`, `--limit`, `--all` cover `nextCursor`/`limit` |
| POST /apps | `container app create` | partial | See flag gaps §2 — repositorySettings, volumes, probes etc. unexposed |
| GET /apps/{appId} | `container app get` (alias: `container get`) | covered | |
| PUT /apps/{appId} (full replace) | — | missing | Client method `update_application` exists but is dead code — no CLI command issues PUT; `app update` uses PATCH. Functionally mostly redundant, but full-replace semantics (e.g. clearing fields, replacing template/volume arrays in one call) are unreachable |
| PATCH /apps/{appId} | `container app update` | partial | name/runtimeType/autoScaling only; `regionSettings` (sibling cmd exists), `containerTemplates` (sibling cmds exist), `volumes` (NOTHING covers — see §2) |
| DELETE /apps/{appId} | `container app delete` (alias: `container delete`) | covered | Plus CLI-only `--cascade`/`--no-cascade` pull-zone handling |
| POST /apps/{appId}/deploy | `container app deploy` | covered | No body |
| POST /apps/{appId}/undeploy | `container app undeploy` | covered | No body |
| POST /apps/{appId}/restart | `container app restart` | covered | No body |
| GET /apps/{appId}/overview | `container app overview` | covered | |
| GET /apps/{appId}/statistics | `container app statistics` | covered | `--from`/`--to`/`--granularity`; granularity required in spec, CLI defaults to `Daily` (sensible) |
| GET /apps/{appId}/summary (Usage Summary) | — | missing | Not in client.rs either. Spec-only op; absent from the hand-written notes |

### Autoscaling & region settings

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /apps/{appId}/autoscaling | `container app autoscaling-get` | covered | |
| PUT /apps/{appId}/autoscaling | `container app autoscaling-update` | covered | `--min`/`--max` (both required, matches spec) |
| GET /apps/{appId}/region-settings | `container app region-settings-get` | covered | |
| PUT /apps/{appId}/region-settings | `container app region-settings-update` | partial | `nodeSelectors` (map<string,string>) hard-coded to `None` in container.rs (~line 1055) |

### Container templates

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| POST /apps/{appId}/containers | `container template add` | partial | Only name + image coords + registry; see §2 |
| GET /apps/{appId}/containers/{containerId} | `container template get` | covered | |
| PATCH /apps/{appId}/containers/{containerId} | `container template update` | partial | Only name + image coords + registry; see §2 |
| DELETE /apps/{appId}/containers/{containerId} | `container template delete` | covered | |
| PUT /apps/{appId}/containers/{containerId}/env | `container template env` | covered | Rich surface: `--add/--update/--remove/--replace-all/--clear/--list` (read-modify-write on the PUT-replace endpoint) |

### Endpoints

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /apps/{appId}/endpoints | `container endpoint list` | covered | |
| POST /apps/{appId}/containers/{containerId}/endpoints | `container endpoint add` | partial | See §2 — SSL, sticky sessions, pullZoneId, protocols unexposed |
| PUT /apps/{appId}/endpoints/{endpointId} | `container endpoint update` | partial | Same gaps as add |
| DELETE /apps/{appId}/endpoints/{endpointId} | `container endpoint delete` | covered | |

### Volumes

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /apps/{appId}/volumes | `container volume list` | covered | |
| PATCH /apps/{appId}/volumes/{volumeId} | `container volume update` | covered | `--name`/`--size` = full body |
| POST /apps/{appId}/volumes/{volumeId}/detach | `container volume detach` | covered | |
| DELETE /apps/{appId}/volumes/{volumeId} | `container volume delete` | covered | |
| DELETE /apps/{appId}/volumes/{volumeId}/instances/{instanceId} | `container volume delete-instance` | covered | |

Note: there is no create-volume endpoint — volumes are created via the `volumes` array on
POST/PUT/PATCH `/apps/{appId}`, which the CLI does not expose. So while all five volume
*endpoints* are covered, the volume *lifecycle* cannot be started from hoppy (see §2/§4).

### Pods, nodes, regions, limits

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| POST /apps/{appId}/pods/{podId}/recreate | `container pod recreate` | covered | |
| GET /nodes | `container node list` | covered | `--cursor`/`--limit`/`--all` |
| GET /nodes/plain (Node IPs, plain) | — | missing | Not in client.rs. Spec-only; absent from notes |
| GET /regions | `container region list` | covered | `--cursor`/`--limit`/`--all` |
| GET /regions/optimal | `container region optimal` | partial | `cdnServerToken` query param unexposed — CLI calls `get_optimal_region(None)` (container.rs ~line 2067) though the client signature accepts it |
| GET /limits | `container limits` | covered | |

### Log forwarding

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /log/forwarding | `container log-forwarding list` | covered | |
| POST /log/forwarding | `container log-forwarding create` | covered | All 7 body fields mapped (`--id` → `app`) |
| GET /log/forwarding/{appId} | `container log-forwarding get` | covered | |
| PUT /log/forwarding/{appId} | `container log-forwarding update` | covered | All fields; `--enabled` footgun — see §2 |
| DELETE /log/forwarding/{appId} | `container log-forwarding delete` | covered | |

### Container registries

| METHOD path | CLI command | Status | Notes |
|---|---|---|---|
| GET /registries | `container registry list` | covered | |
| POST /registries | `container registry create` | covered | `--name/--registry-type/--username/--password` = full body. Op is absent from the notes (spec-only) |
| GET /registries/{registryId} | `container registry get` | covered | |
| PUT /registries/{registryId} | `container registry update` | covered | Full body |
| DELETE /registries/{registryId} | `container registry delete` | covered | |
| POST /registries/images (List Container Images) | — | missing | Client method `list_container_images` EXISTS but no CLI command exposes it (`registry` has no `images` subcommand) |
| POST /registries/tags | `container registry image-tags` | covered | Full body |
| POST /registries/digest | `container registry image-digest` | covered | Full body |
| POST /registries/config-suggestions | `container registry config-suggestions` | covered | Full body |
| POST /registries/image-config (Get Image Config) | — | missing | Not in client.rs, not in notes. Spec-only op |
| POST /registries/public-images/search | `container registry search-public` | covered | `--query` → `prefix`, plus `--size`/`--page` |

## 2. Flag-level gaps per command

### `container app create` (POST /apps)

| Documented property | Flag | Status |
|---|---|---|
| name | `--name` | ok |
| runtimeType | `--runtime-type` | ok (spec enum is lowercase `shared`/`reserved`; CLI help says "Shared or Reserved" — casing handled by parse, worth a deserialize test) |
| autoScaling.min / .max | `--min` / `--max` | ok |
| regionSettings.allowedRegionIds | `--region` (repeatable) | ok |
| regionSettings.requiredRegionIds | — | MISSING (workaround: `region-settings-update` after create) |
| regionSettings.maxAllowedRegions | — | MISSING (same workaround) |
| regionSettings.nodeSelectors | — | MISSING (no workaround anywhere) |
| terminationGracePeriodSeconds (1-300) | — | MISSING |
| repositorySettings {templateRepository, repositoryName, owner} | — | MISSING |
| containerTemplates[].imageName/Namespace/Tag/RegistryId | `--image-name/--image-namespace/--image-tag/--registry-id` | ok (single template only) |
| containerTemplates[].imageDigest | — | MISSING (hard-coded `None`) |
| containerTemplates[].imagePullPolicy | — | MISSING (hard-coded `IfNotPresent`; API enum also has `Always`) |
| containerTemplates[].entryPoint {command, commandArray, arguments, argumentsArray, workingDirectory} | — | MISSING |
| containerTemplates[].probes {startup, readiness, liveness} | — | MISSING |
| containerTemplates[].environmentVariables | `--env` | ok-ish (applied via follow-up `template env --replace-all`, not in the create body) |
| containerTemplates[].endpoints | — | MISSING at create (workaround: `endpoint add` after) |
| containerTemplates[].volumeMounts | — | MISSING (no workaround) |
| volumes[] {name, size} | — | MISSING (no workaround — volumes cannot be created via CLI at all) |
| multiple containerTemplates | — | MISSING (exactly 0 or 1; workaround: `template add` after) |

### `container app update` (PATCH /apps/{appId})

| Documented property | Flag | Status |
|---|---|---|
| name | `--name` | ok |
| runtimeType | `--runtime-type` | ok |
| autoScaling | `--min`/`--max` | ok (read-modify-write preserves the other bound) |
| regionSettings | — | absent here; covered by `app region-settings-update` |
| containerTemplates | — | absent here; covered by `template` subcommands |
| volumes | — | MISSING everywhere (see above) |
| PUT-only: terminationGracePeriodSeconds, repositorySettings | — | MISSING (PUT itself is unreachable from CLI) |

### `container app statistics` (GET .../statistics)

All query params exposed (`fromDate`→`--from` required, `toDate`→`--to`, `granularity`→`--granularity`
defaulting to Daily where spec marks it required). No gaps.

### `container app region-settings-update` (PUT .../region-settings)

| Documented property | Flag | Status |
|---|---|---|
| allowedRegionIds | `--allowed-region` | ok |
| requiredRegionIds | `--required-region` | ok |
| maxAllowedRegions | `--max-allowed-regions` | ok |
| nodeSelectors | — | MISSING (hard-coded `None`) |

### `container template add` (POST .../containers)

| Documented property | Flag | Status |
|---|---|---|
| name, imageName, imageNamespace, imageTag, imageRegistryId | `--name/--image-name/--image-namespace/--image-tag/--registry-id` | ok |
| image | — | MISSING (nullable convenience field; low value) |
| imageDigest (`sha256:...`) | — | MISSING |
| imagePullPolicy (`Always`/`IfNotPresent`) | — | MISSING (sent as `None`) |
| entryPoint (5 sub-fields) | — | MISSING |
| probes (startup/readiness/liveness × httpGet/tcpSocket/grpc + thresholds) | — | MISSING |
| environmentVariables | — | missing at create; covered by `template env` afterwards |
| endpoints | — | missing at create; covered by `endpoint add` afterwards |
| volumeMounts {name, mountPath} | — | MISSING (no workaround) |

### `container template update` (PATCH .../containers/{containerId})

Same gaps as `template add`: `--name/--image-name/--image-namespace/--image-tag/--registry-id`
exposed; imageDigest, imagePullPolicy, entryPoint, probes, volumeMounts unpatchable.
environmentVariables/endpoints covered by sibling commands.

### `container endpoint add` / `container endpoint update`

| Documented property | Flag | Status |
|---|---|---|
| displayName | `--name` | ok |
| cdn vs anycast selection | `--cdn` / `--anycast` (default CDN) | ok |
| portMappings[].containerPort | `--container-port` | ok |
| portMappings[].exposedPort | `--exposed-port` | ok |
| portMappings[].protocols (`Tcp`/`Udp`/`Sctp`) | — | MISSING (sent as `None`) |
| multiple portMappings | — | MISSING for anycast (spec allows >1 there; CDN is capped at exactly 1, so single mapping is correct for CDN) |
| cdn.isSslEnabled | — | MISSING (hard-coded `None`) |
| cdn.stickySessions {enabled, sessionHeaders (1-3, required), cookieName} | — | MISSING |
| cdn.pullZoneId | — | MISSING |
| anycast.type | hard-coded `IPv4` | benign — spec enum has only `IPv4` |

### `container region optimal` (GET /regions/optimal)

| Documented property | Flag | Status |
|---|---|---|
| cdnServerToken (query) | — | MISSING — client accepts `Option<&str>` but CLI passes `None` |

### `container log-forwarding create` / `update` (POST / PUT /log/forwarding)

All body fields exposed: `app`→`--id`, `type`→`--forwarding-type`, `endpoint`, `port`,
`token`, `format`→`--syslog-format`, `enabled`→`--enabled`. One usability footgun:
`enabled` is a required boolean in the body and `--enabled` is a bare flag, so running
`update` without `--enabled` silently disables forwarding (no `--no-enabled`/tri-state).

### Fully clean commands (all documented inputs exposed)

`app list/get/deploy/undeploy/restart/delete/overview/autoscaling-get/autoscaling-update/region-settings-get`,
`template get/delete/env`, `endpoint list/delete`, `volume list/update/detach/delete/delete-instance`,
`registry list/get/create/update/delete/image-tags/image-digest/config-suggestions/search-public`,
`region list`, `node list`, `pod recreate`, `limits`, `log-forwarding list/get/delete`.

## 3. CLI-only surface

Commands/flags with no documented API counterpart (none of these hit undocumented MC endpoints
— every URL in client.rs matches a spec operation):

- `container logs` — composite convenience: binds a local TCP syslog listener, opens a bore
  tunnel (or uses `--tunnel none`/`--tunnel-host`), creates a *temporary* log-forwarding config
  via POST /log/forwarding, streams, then deletes it. Flags `--tunnel`, `--tunnel-host`,
  `--local-port`, `--replace-existing`, `--follow` (no-op), `--bore-server` are all CLI-side.
  There is no log-streaming endpoint in the MC API; this is built entirely on log forwarding.
  Known issue: empty-body 400 at the create step (backlog/log-forwarding-create-empty-400.md).
- `container list` / `container get` / `container delete` — pure aliases for `container app ...`.
- `container app delete --cascade` / `--no-cascade` — cross-service behavior: discovers
  auto-managed Pull Zones from endpoint `pullZoneId`s and deletes them via the *core platform*
  Pull Zone API. Not an MC API feature.
- `container app create --env` — implemented as a follow-up PUT .../env after create, not a body property.
- `container app create --minimal` — output shaping (legacy `{"id": ...}` response).
- `container template env --list` — client-side read of GET .../containers/{id} (no list-env endpoint).
- Global flags `--reveal`, `--reveal-env`, `--no-hints`, `--record`, `--no-redact`, `--minimal` — CLI-side plumbing.

Client-only surface (implemented in client.rs, unreachable from the CLI):
- `update_application` → PUT /apps/{appId} (CLI uses PATCH instead)
- `list_container_images` → POST /registries/images (no `registry images` subcommand)

## 4. Observations

1. **Fresh spec > notes.** The notes (fetched 2026-03-18) are missing six operations the fresh
   spec documents: GET /apps (list), POST /apps (add), GET /apps/{appId}/summary,
   GET /nodes/plain, POST /registries (create), POST /registries/image-config. Coverage claims
   based on notes alone would have over-counted coverage (e.g. would not have caught
   summary/nodes-plain/image-config as missing). The applications note documents Get/Overview/
   Statistics/Patch/Put/Deploy/Undeploy/Restart/Delete but not List/Add.
2. **Enum casing drift between spec and notes/CLI.** Spec: `runtimeType` = `shared`/`reserved`,
   registry `type` = `dockerHub`/`gitHub` (camelCase); notes and CLI help use PascalCase
   (`Shared`, `DockerHub`). Given this project's history of casing bugs (iter-48 geo-zone fix,
   iter-65 toggle-casing tests), the serde rename behavior for these two enums is worth a
   deserialize/serialize round-trip test against the spec casing.
3. **Volume lifecycle is one-way.** The API only creates volumes through the `volumes` array on
   POST/PUT/PATCH /apps and only mounts them through `volumeMounts` on container requests. The
   CLI exposes neither, so `volume list/update/detach/delete` can only operate on volumes created
   via the bunny.net dashboard. This is the largest functional hole despite every volume
   *endpoint* being "covered".
4. **Probes/entryPoint/pull-policy are a deep, deliberate-looking cut.** Container.rs hard-codes
   these to `None` (and `IfNotPresent` for create). They are large nested schemas (3 probe types ×
   http/tcp/grpc); a `--probes-json`/`--entrypoint` style escape hatch may be cheaper than full
   flag mapping.
5. **`GET /apps/{appId}/summary` uncertainty.** The spec documents it with no response schema
   detail beyond the summary name ("Get Application Usage Summary"); the notes don't mention it.
   Verify against the live API before implementing (dogfooding candidate).
6. **Coverage claims are high-confidence** for everything in §1: every client.rs method carries a
   doc comment with its route, and all 41 routes were matched 1:1 against the spec inventory; CLI
   handlers were spot-verified in container.rs for every "partial" judgment (hard-coded `None`s
   located at lines ~744-771, ~1055, ~1236-1242, ~1552-1576, ~2067).

## Summary counts

- Total documented operations (fresh spec, authoritative): **52**
- Covered (CLI command, all documented inputs exposed): **39**
- Partial (CLI command exists, documented properties missing): **8**
  (app create, app update/PATCH, region-settings-update, template add, template update, endpoint add, endpoint update, region optimal)
- Missing (no CLI path): **5**
  (PUT /apps/{appId} full replace; GET /apps/{appId}/summary; GET /nodes/plain; POST /registries/images — client method exists but unexposed; POST /registries/image-config)
- 5 most impactful gaps:
  1. Volumes cannot be created or mounted via CLI — `volumes` (app create/update) and `volumeMounts` (template add/update) are unexposed, making the whole volume subcommand group usable only on dashboard-created volumes.
  2. Container runtime configuration unexposed — `entryPoint`, `probes` (startup/readiness/liveness), `imagePullPolicy`, `imageDigest` on template add/update; no health checks or custom commands from hoppy.
  3. CDN endpoint options unexposed — `isSslEnabled`, `stickySessions`, `pullZoneId`, `protocols` on endpoint add/update.
  4. `registry images` command absent — POST /registries/images is already implemented in client.rs (`list_container_images`) but has no CLI surface (cheap win).
  5. Three endpoints absent from the client entirely — GET /apps/{appId}/summary, GET /nodes/plain, POST /registries/image-config (spec-only; also missing from the KB notes, which should be refreshed).
