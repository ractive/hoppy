---
title: Iter-76 — containers polish (volumes, probes, endpoints)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - containers
status: planned
branch: iter-76/containers-polish
priority: 4
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/containers
---

# Iter-76 — containers polish

## Why

Per [[research/api-coverage-2026-07/containers]], the volume lifecycle is
one-way (only dashboard-created volumes are manageable), container
runtime config (probes/entrypoint/pull policy) is hardcoded to `None`,
and CDN endpoint options are unexposed. One command is a pure wiring win
(`registry images` — client method already exists).

## Scope

### 1. Volume lifecycle

- [ ] Expose `volumes[] {name, size}` on `container app create` /
  `app update` (`POST`/`PATCH /apps`) — volumes become creatable from CLI
- [ ] Expose `volumeMounts {name, mountPath}` on `container template
  add` / `template update`

### 2. Container runtime config

- [ ] `probes` (startup/readiness/liveness × httpGet/tcpSocket/grpc) on
  template add/update — prefer a `--probes-json <file>` escape hatch
  over full flag mapping (report §4 recommendation)
- [ ] `entryPoint` (command, commandArray, arguments, argumentsArray,
  workingDirectory)
- [ ] `imagePullPolicy` (`Always`/`IfNotPresent`; currently hardcoded)
  and `imageDigest` (`sha256:…`)

### 3. Endpoint options

- [ ] `isSslEnabled`, `stickySessions {enabled, sessionHeaders,
  cookieName}`, `pullZoneId`, `portMappings[].protocols`
  (`Tcp`/`Udp`/`Sctp`) on `container endpoint add` / `update`; allow
  multiple port mappings for anycast endpoints

### 4. Registry images command

- [ ] `container registry images` → `POST /registries/images` — client
  method `list_container_images` exists, wiring only

### 5. Missing endpoints

- [ ] `container app summary` → `GET /apps/{appId}/summary` (live-verify
  the response shape first — no schema in spec, report §4.5)
- [ ] `container node ips` (or `node list --plain`) → `GET /nodes/plain`
- [ ] `container registry image-config` → `POST /registries/image-config`

### 6. PUT full-replace decision

- [ ] Decide whether to expose `update_application` (PUT `/apps/{appId}`,
  dead client code) as e.g. `container app replace`, or document
  PATCH-only; record the outcome in [[decision-log]]

## Out of scope

- `regionSettings.nodeSelectors`, `repositorySettings`,
  `terminationGracePeriodSeconds` — backlog
- `container region optimal --cdn-server-token` — pair with the db
  `cdn_server_token` work if it lands in [[iteration-66-spec-refresh-drift-fixes]]
- `log-forwarding update --enabled` tri-state footgun — backlog

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated for all new commands/flags
- [ ] `hyalo lint` clean on touched knowledgebase files
