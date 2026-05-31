---
title: "Spec coverage audit — container-app"
type: research
date: 2026-05-31
tags:
  - audit
  - openapi
  - spec-coverage
  - todo
---

# Spec coverage audit — `container-app`

> **TODO — no spec file available.** The bunny.net Magic Containers
> (Container App) API is not currently checked into `specs/`. The
> hand-written `containers/types.rs` module models ~15 request/response
> shapes (`ContainerInstance`, `ContainerTemplate`, `ContainerRequest`,
> `ContainerEndpoint`, …) without a reference document to diff against.
>
> Follow-up actions:
>
> - Locate / export the Magic Containers OpenAPI spec from bunny.net.
> - Drop it under `specs/containers.json`.
> - Re-run `run-spec-coverage-audit.sh` after extending the tuple list
>   in the orchestrator to include each `Container*` struct.
