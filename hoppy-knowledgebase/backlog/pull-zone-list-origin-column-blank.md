---
title: pull-zone list Origin column is blank or meaningless
type: backlog
date: 2026-10-23
status: completed
priority: medium
origin: dogfooding-2026-10-23
---

# `hoppy pull-zone list` Origin column shows nothing useful

Live data (2026-10-23): ordinary zones have `OriginType` 2 (StorageZone), `OriginUrl` `""` and a `StorageZoneId` such as 1520401, so the column is blank. Magic Containers zones have `OriginType` 5, the placeholder `OriginUrl` `https://bunny.net` and `StorageZoneId` -1.

Fix: derive the label from the origin type (storage zone name via one list call, container app name, URL, or a type label). JSON output keeps the raw API shape. Fixed in [[iterations/iteration-88-pull-zone-origin-and-id-hints]].
