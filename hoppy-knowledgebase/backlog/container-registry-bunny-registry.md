---
title: bunnyRegistry registry type, image find and paged tags not covered
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - containers
  - registries
  - api-coverage
---

# Magic Containers registry additions

- `RegistryType` / `ContainerRegistryRequest.type` gained `bunnyRegistry`
- `ContainerRegistry.isPlatformManaged` (bool)
- `POST /registries/images/find` (`FindContainerImageRequest { image, registryIds }`)
- `POST /registries/tags/paged` (cursor-paged tag listing)

hoppy: `RegistryType` (`crates/bunny-net-api/src/containers/types.rs:232`) has
only `DockerHub`/`GitHub`. **Pre-existing issue that needs a live check:** the
enum has no `rename_all`, so it serializes as `DockerHub`/`GitHub`, while the
spec says `dockerHub`/`gitHub`.

- [ ] Verify the casing the live API accepts and fix the serialization if needed
- [ ] Add `BunnyRegistry`, `is_platform_managed`, and the two new endpoints

See [[research/openapi-drift-2026-10-23]].

## Progress (iter-87, 2026-10-23)

- `RegistryType` now has a `BunnyRegistry` read variant and accepts camelCase or PascalCase on responses (also for the two registry status enums).
- **Casing on requests is still unverified.** `registry list` returns no `type` field, and exercising `registry add` needs a write, so hoppy keeps sending PascalCase (`DockerHub`). A test pins that. Verify against a real `registry add` before changing it.
- Still open: `is_platform_managed`, `POST /registries/images/find`, `POST /registries/tags/paged`.
