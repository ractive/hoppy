---
title: storage-zone regions likely prints empty columns after StorageRegionModel reshape
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - storage-zone
  - api-drift
  - dogfooding
---

# `StorageRegionModel` reshaped

The spec removed `Id` and `Url`. It added `Code`, `Endpoint`, `S3Endpoint`,
`Continent`, `Latitude`, `Longitude`, `PricePerGigabyte`, `Tier`,
`IsMainRegionSupported` and `IsS3Supported`. `core::StorageRegion`
(`crates/bunny-net-api/src/core/types.rs:4711`) has only optional `id`, `name`
and `url`, so a live response would deserialize with `id`/`url` = `None`
without any error.

The fixture `fixtures/core/storagezone_regions.json` still has the July shape,
so tests stay green either way.

- [ ] Dogfood `GET /storagezone/regions` and confirm the live shape
- [ ] If confirmed, model the new fields and update the fixture, client type and test together, inside an iteration (per decision 2026-07-10)
- [ ] Surface `Code`, `Endpoint` and `S3Endpoint` in the CLI table

See [[research/openapi-drift-2026-10-23]].
