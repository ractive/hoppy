---
title: storage-zone regions likely prints empty columns after StorageRegionModel reshape
type: backlog
date: 2026-10-23
status: completed
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - storage-zone
  - api-drift
  - dogfooding
resolved-in: iter-87
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

- [x] Dogfood `GET /storagezone/regions` and confirm the live shape
- [x] If confirmed, model the new fields and update the fixture, client type and test together, inside an iteration (per decision 2026-07-10)
- [x] Surface `Code`, `Endpoint` and `S3Endpoint` in the CLI table

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

Live `GET /storagezone/regions` confirmed the new shape (24 regions). `StorageRegion` now models Code, Name, Continent, Latitude, Longitude, PricePerGigabyte, Tier, IsMainRegionSupported, IsS3Supported, Endpoint, S3Endpoint; fixture, type and tests were updated together; the table shows Code, Name, Endpoint, S3 Endpoint.

See [[iterations/iteration-87-openapi-drift-fixes]].
