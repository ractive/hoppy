---
title: "Iteration 0.5 — Codegen Experiment"
type: iteration
date: 2026-03-17
tags:
  - iteration
  - codegen
  - progenitor
  - experiment
status: completed
branch: iter-0.5/codegen-experiment
---

# Iteration 0.5 — Codegen Experiment

**Goal:** Validate whether Progenitor can generate usable Rust clients from the bunny.net OpenAPI specs. Determine the codegen strategy before writing service implementations.

## OpenAPI Specs Inventory

All specs are **OpenAPI 3.0.x** (no Swagger 2.0):

| API | Spec URL | OAS Version | Endpoints | Schemas | Base URL |
|-----|----------|-------------|-----------|---------|----------|
| Core Platform | [public.json](https://core-api-public-docs.b-cdn.net/docs/v3/public.json) | 3.0.0 | ~65 | ~100 | `api.bunny.net` |
| Stream (Video) | [bunnynet-video-api.public.json](https://video.bunnycdn.com/openapi/bunnynet-video-api.public.json) | 3.0.0 | ~30 | ~50 | `video.bunnycdn.com` |
| Shield | [swagger.json](https://api.bunny.net/shield/docs/v1/swagger.json) | 3.0.4 | ~41 | ~60 | `api.bunny.net/shield` |
| Edge Scripting | [compute.json](https://core-api-public-docs.b-cdn.net/docs/v3/compute.json) | 3.0.0 | ~22 | ~24 | `api.bunny.net/compute` |
| Storage | [openapi.json](https://docs.bunny.net/api-reference/storage/openapi.json) | 3.0.0 | 4 | 2 | `{region}.storage.bunnycdn.com` |

## Experiment Plan

- [x] Download all 5 OpenAPI spec files into `specs/` directory
- [x] Install `cargo-progenitor`
- [x] Run Progenitor against each spec, record results:
  - [x] Core Platform — largest, most important
  - [x] Stream — second priority
  - [x] Shield — uses 3.0.4, may have quirks
  - [x] Edge Scripting — smaller, good test case
  - [x] Storage — tiny, baseline test
- [x] For each spec, evaluate:
  - Does it generate without errors?
  - Do the generated types look correct?
  - Does the generated client compile?
  - Are the method signatures usable?
- [x] If Progenitor fails on a spec, try minor spec fixes (remove unsupported features, fix schema issues)
- [x] If Progenitor fails fundamentally, try `openapi-generator` as fallback
- [x] Document results and decide strategy per API

## Expected Outcome

A decision matrix:

| API | Codegen? | Tool | Notes |
|-----|----------|------|-------|
| Core Platform | yes/no | progenitor / openapi-generator / hand-written | ... |
| Stream | yes/no | ... | ... |
| Shield | yes/no | ... | ... |
| Edge Scripting | yes/no | ... | ... |
| Storage | yes/no | ... | ... |

All specs get the same treatment — if codegen works, we use it for all 5, including Storage. Consistency over convenience.

## Integration Approach

If codegen works, the generated clients go into a workspace member crate:

```
hoppy/
  Cargo.toml          (workspace)
  crates/
    hoppy-cli/        (the CLI binary)
    bunny-api-core/   (Core Platform API client)
    bunny-api-stream/ (Stream API client)
    bunny-api-shield/ (Shield API client)
    bunny-api-compute/(Edge Scripting API client)
    bunny-api-storage/(Edge Storage API client)
  specs/              (downloaded OpenAPI JSON files)
```

The CLI crate depends on the generated crates and wraps their clients with our auth/output/error handling.

**Deliverable:** Decision document with codegen results per spec. Generated crates compile (or documented reasons why not).

## Related
- [[development-roadmap]] — project roadmap
- [[research/openapi-codegen-rust]] — codegen research
- [[research/hand-written-experiment-results]] — experiment results that led to hand-written decision
- [[decision-log]] — decision to abandon codegen
