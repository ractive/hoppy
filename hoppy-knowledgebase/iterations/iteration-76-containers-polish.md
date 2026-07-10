---
title: Iter-76 — containers polish (volumes, probes, endpoints)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - containers
status: in-progress
branch: iter-76/containers-polish
priority: 4
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/containers
---

# Iter-76 — containers polish

> [!note] Carried forward from iter-75
> Iter-75 (account/billing) shipped clean with no spillover into this
> scope — no shared code paths, no API quirks discovered that affect
> containers. One reusable pattern worth reusing here: the streamed
> binary-download helper (`CoreClient::stream_to_writer`, added in
> iter-75 for invoice PDFs) is the template to follow if any iter-76
> endpoint needs to stream a binary body — don't buffer whole payloads
> (project streaming rule). No other scope changes needed.

## Why

Per [[research/api-coverage-2026-07/containers]], the volume lifecycle is
one-way (only dashboard-created volumes are manageable), container
runtime config (probes/entrypoint/pull policy) is hardcoded to `None`,
and CDN endpoint options are unexposed. One command is a pure wiring win
(`registry images` — client method already exists).

## Scope

### 1. Volume lifecycle

- [x] Expose `volumes[] {name, size}` on `container app create` /
  `app update` (`POST`/`PATCH /apps`) — volumes become creatable from CLI
- [x] Expose `volumeMounts {name, mountPath}` on `container template
  add` / `template update`

### 2. Container runtime config

- [x] `probes` (startup/readiness/liveness × httpGet/tcpSocket/grpc) on
  template add/update — prefer a `--probes-json <file>` escape hatch
  over full flag mapping (report §4 recommendation)
- [x] `entryPoint` (command, commandArray, arguments, argumentsArray,
  workingDirectory)
- [x] `imagePullPolicy` (`Always`/`IfNotPresent`; currently hardcoded)
  and `imageDigest` (`sha256:…`)

### 3. Endpoint options

- [x] `isSslEnabled`, `stickySessions {enabled, sessionHeaders,
  cookieName}`, `pullZoneId`, `portMappings[].protocols`
  (`Tcp`/`Udp`/`Sctp`) on `container endpoint add` / `update`; allow
  multiple port mappings for anycast endpoints

### 4. Registry images command

- [x] `container registry images` → `POST /registries/images` — client
  method `list_container_images` exists, wiring only

### 5. Missing endpoints

- [x] `container app summary` → `GET /apps/{appId}/summary` (live-verify
  the response shape first — no schema in spec, report §4.5)
- [x] `container node ips` (or `node list --plain`) → `GET /nodes/plain`
- [x] `container registry image-config` → `POST /registries/image-config`

### 6. PUT full-replace decision

- [x] Decide whether to expose `update_application` (PUT `/apps/{appId}`,
  dead client code) as e.g. `container app replace`, or document
  PATCH-only; record the outcome in [[decision-log]]

## Out of scope

- `regionSettings.nodeSelectors`, `repositorySettings`,
  `terminationGracePeriodSeconds` — backlog
- `container region optimal --cdn-server-token` — pair with the db
  `cdn_server_token` work if it lands in [[iteration-66-spec-refresh-drift-fixes]]
- `log-forwarding update --enabled` tri-state footgun — backlog

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated for all new commands/flags
- [x] `hyalo lint` clean on touched knowledgebase files
