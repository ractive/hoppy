---
title: stream resolutions cleanup discards the new cleanup report (dry-run shows nothing)
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - stream
  - api-drift
---

# `POST …/resolutions/cleanup` now returns a cleanup plan

Stream spec 1.6.12 changes the 200 (and 400) response from `StatusModel` to
`StatusModelOfResolutionsCleanupModel`, which wraps `ResolutionsCleanupModel`.
That model has `resolutionsToDelete`, `resolutionsToDeleteFromPlaylist`,
`…FromStorage`, `…FromMp4`, `storageObjectsToDelete`,
`availableResolutionsAfter`, `modifiedPlaylist`, `deletedStorageObjects` and
`deletedOriginal`. `cleanup_video_resolutions`
(`crates/bunny-net-api/src/stream/client.rs:584`) returns `StatusMessage` and
drops all of it. As a result, the server-side `--dry-run` preview tells the
user nothing.

- [ ] Model `ResolutionsCleanupModel` and return it
- [ ] Render the plan in the CLI, especially for `--dry-run`

See [[research/openapi-drift-2026-10-23]].
