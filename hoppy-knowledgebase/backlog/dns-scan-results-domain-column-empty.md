---
title: dns zone scan results --domain shows blank Domain column
type: backlog
date: 2026-06-01
status: planned
priority: low
origin: dogfooding-2026-06-01
tags:
  - cli
  - dns
  - table
  - polish
---

# `dns zone scan results --domain X` doesn't fill the Domain column

After iter-53 added `--domain` to `dns zone scan results`, the
command resolves the zone id client-side via the zone list and then
calls the existing scan-results endpoint. The table output:

```sh
$ hoppy dns zone scan results --domain hoppy-test-1780305804.hoppy.test
+--------------------------------------+---------+--------+------------+---------+----------------------------------+-----------+
| Job ID                               | Zone ID | Domain | Status     | Records | Created                          | Completed |
+--------------------------------------+---------+--------+------------+---------+----------------------------------+-----------+
| 2ae95549-5c19-4da2-b63e-477e45e16a7e | 803142  | -      | InProgress | 0       | 2026-06-01T09:23:36.127043+00:00 | -         |
+--------------------------------------+---------+--------+------------+---------+----------------------------------+-----------+
```

The `Domain` column shows `-` even though the user just told us the
domain on the command line, and we used it to resolve the zone id.

## Fix

In the `--domain` code path, plumb the user-supplied domain (or the
resolved domain from the zone-list lookup) into the rendered row so
`Domain` shows the actual value. JSON output should already include
it once the field is on the model.

Cheap and a small DX win.

## Acceptance

- `dns zone scan results --domain X` table shows `X` in the Domain
  column.
- `dns zone scan results --id <z>` looks up the domain via the zone
  list and shows it too (consistency).
- `--format json` includes the `Domain` field.
