---
title: OpenAPI Code Generation for Rust - Evaluation
date: 2026-03-17
tags:
  - rust
  - openapi
  - codegen
  - progenitor
  - api-client
status: completed
recommendation: codegen-with-progenitor
type: research
---

# OpenAPI Code Generation for Rust

## Options Evaluated

### 1. Progenitor (Oxide Computer)

**Repository:** <https://github.com/oxidecomputer/progenitor>
**License:** Apache-2.0 / MIT
**Maintained:** Yes (actively maintained by Oxide)

**Features:**
- Generates opinionated Rust clients from OpenAPI 3.0.x specs
- Async via Rust futures, paginated interfaces via Streams
- Built on reqwest 0.13
- Two interface styles: positional and builder pattern
- Can generate CLI and httpmock helpers
- WebSocket endpoint support
- Three usage modes:
  1. **Macro** (`generate_api!`) - inline generation
  2. **build.rs** - generates visible code, supports CLI/mock generation
  3. **cargo-progenitor** - standalone crate generation

**Pros:**
- High quality Rust code
- Builder pattern is ergonomic
- Active maintenance from a reputable Rust shop (Oxide)
- Can generate a CLI directly (useful for prototyping)

**Caveats:**
- Primary target is Dropshot-generated APIs
- "May fail for some OpenAPI documents" - not all specs are supported
- Bunny.net specs may need preprocessing/fixing

### 2. OpenAPI Generator (openapi-generator-cli)

**Repository:** <https://github.com/OpenAPITools/openapi-generator>
**Maintained:** Yes (large community)

**Features:**
- Supports OpenAPI v2 and v3
- Generates Rust client using `reqwest` or `hyper`
- Wide language support (50+)
- Java-based tool (requires JRE)

**Pros:**
- Very mature, large community
- Broad OpenAPI spec compatibility
- Well-documented

**Cons:**
- Generated Rust code is often not idiomatic
- Java dependency for code generation
- Generated code may need significant cleanup
- Less Rust-specific optimization

### 3. utoipa + Manual Client

**Alternative approach:** Don't generate a client at all. Instead, use the OpenAPI specs as reference and hand-write a thin client layer using reqwest directly.

**Pros:**
- Full control over code quality
- No dependency on generator tooling
- Easier to customize for bunny.net specifics

**Cons:**
- More initial work
- Must manually keep in sync with API changes

## Bunny.net Specific Considerations

Bunny.net has **5 separate OpenAPI specs** (all verified OpenAPI 3.0.x):

| API | Spec URL | OAS Version | Endpoints | Schemas |
|-----|----------|-------------|-----------|---------|
| Core Platform | `core-api-public-docs.b-cdn.net/docs/v3/public.json` | 3.0.0 | ~65 | ~100 |
| Stream (Video) | `video.bunnycdn.com/openapi/bunnynet-video-api.public.json` | 3.0.0 | ~30 | ~50 |
| Shield | `api.bunny.net/shield/docs/v1/swagger.json` | 3.0.4 | ~41 | ~60 |
| Edge Scripting | `core-api-public-docs.b-cdn.net/docs/v3/compute.json` | 3.0.0 | ~22 | ~24 |
| Storage | `docs.bunny.net/api-reference/storage/openapi.json` | 3.0.0 | 4 | 2 |

Despite the filename "swagger.json", Shield is actually **OpenAPI 3.0.4** — not Swagger 2.0.

Remaining complexity:
1. Different base URLs per API
2. Authentication varies (main key vs per-service keys)
3. Each spec may have different conventions

## Recommendation: Hybrid Approach

**Use Progenitor for initial type/client generation, then customize.**

1. **Download all OpenAPI specs** and convert Shield's Swagger 2.0 to OpenAPI 3.0
2. **Try Progenitor** on each spec - it will likely work for most
3. **For specs that fail**, fall back to hand-writing those clients using the generated ones as reference
4. **Wrap generated clients** in a unified interface that handles:
   - Different base URLs
   - Authentication routing (main key vs service-specific keys)
   - Consistent error handling
   - Output formatting

**Alternative simpler approach:** Given the complexity of 6 different specs, it may be more practical to **hand-write the client** using reqwest directly with serde types derived from the OpenAPI specs. Start with the most important endpoints (pull zones, storage, DNS) and expand from there.

### Quick Start Commands

```bash
# Install progenitor
cargo install cargo-progenitor

# Try generating from core API spec
cargo progenitor \
  -i core-api-public.json \
  -o src/client/core \
  -n bunny-core \
  -v 0.1.0

# Try generating from stream API spec
cargo progenitor \
  -i bunnynet-video-api.public.json \
  -o src/client/stream \
  -n bunny-stream \
  -v 0.1.0
```

## Sources

- [Progenitor GitHub](https://github.com/oxidecomputer/progenitor)
- [Progenitor Docs](https://docs.rs/progenitor)
- [OpenAPI Generator - Rust](https://openapi-generator.tech/docs/generators/rust/)
- [OpenAPI Generator GitHub](https://github.com/OpenAPITools/openapi-generator)

## Related

- [[research/hand-written-experiment-results]] — results of hand-written approach
- [[decision-log]] — decision to use hand-written clients
- [[Seed]] — original project brief mentioning codegen
