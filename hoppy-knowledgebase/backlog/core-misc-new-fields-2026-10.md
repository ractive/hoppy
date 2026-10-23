---
title: Video library geo-blocking, live-stream counts and storage zone S3Hostname not modeled
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - video-library
  - storage-zone
  - api-coverage
---

# Core miscellaneous field additions

- `VideoLibraryModel.GeoBlocking` (`Enabled`, `AllowedCountries`, `BlockedCountries`), settable through `VideoLibraryUpdateModel.GeoBlocking`
- `VideoLibraryModel.LiveStreamCount`, `TotalCount`
- `StorageZoneModel.S3Hostname`

None are modeled in `crates/bunny-net-api/src/core/types.rs`.

- [ ] Model the fields and add `video-library update --geo-allow/--geo-block`
- [ ] Show `S3Hostname` in `storage-zone get`

See [[research/openapi-drift-2026-10-23]].
