---
title: db delete / db group delete print empty tables instead of confirmation
type: backlog
date: 2026-05-10
status: completed
priority: medium
origin: dogfooding-2026-05-10
---

# `db delete` doesn't say what it did

```sh
hoppy db delete --id <db-id> --yes
# +----------------------+
# | Status               |
# +----------------------+
hoppy db group delete --id <group-id> --yes
# +----------------------------------+------+---------+---------+----------+
# | ID                               | Name | Storage | Primary | Replicas |
# +----------------------------------+------+---------+---------+----------+
```

Both deletes print the **header row of an empty table** with no body.
Compare with peers:

- `pull-zone delete --id N --yes` → `Deleted pull zone N`
- `dns zone delete --id N --yes`  → `Deleted DNS zone N`
- `container delete --id X --yes` → `Deleted application X`
- `storage-zone delete --id N --yes` → `Deleted storage zone N`
- `stream library delete --id N --yes` → `Deleted video library N`

`db delete` should print `Deleted database <id>` (text format) and
`{"deleted": "<id>"}` for `--format json`. `db group delete` should print
`Deleted database group <id>` (text format) and `{"deleted": "<id>"}` for
`--format json`. Iter-20 follow-up.
