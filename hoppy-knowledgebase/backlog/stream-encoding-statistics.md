---
title: Stream encoding statistics endpoint and new video/collection fields not covered
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - stream
  - api-coverage
---

# Stream 1.5.6 → 1.6.12 additions

- `GET /library/{libraryId}/statistics/encoding` (`VideoEncodingStatisticsModel`: hourly, total, premium, free; breakdowns by codec, resolution and JIT)
- `CollectionModel.liveStreamCount`
- `VideoPlayDataModel`: `pullZoneId`, `maxSoftwareWidevineResolution`, `widevineEmeRobustness`, `widevineSdOnlyForL3`
- `VideoOEmbedModel`: `thumbnail_width`, `thumbnail_height`
- `CaptionModelAdd.srclang` is now **deprecated** in the body, because the path parameter carries it. hoppy still sends it (`crates/bunny-net-api/src/stream/types.rs:55`). This is harmless for now; make it optional when it is next touched.

- [ ] Add `stream statistics encoding`
- [ ] Model the new fields

See [[research/openapi-drift-2026-10-23]].
