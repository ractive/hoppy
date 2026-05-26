---
title: "storage CLI is missing zone management (only has file ops)"
type: backlog
date: 2026-05-26
status: planned
priority: medium
origin: dogfooding-2026-05-26 (post-iter-38)
---

# `hoppy storage` has no zone management subcommands

`hoppy storage --help` currently exposes only file ops:

```
upload    Upload a file
download  Download a file
ls        List files
rm        Delete a file
```

A user with no existing storage zone cannot bootstrap one through hoppy —
they have to use the dashboard. Pull zones, DNS zones, video libraries,
shield zones, and magic container apps all have full `list/get/create/
update/delete` surface. Storage zones do not.

## Suggested shape

```
hoppy storage zone list
hoppy storage zone get --id <id> | --name <name>
hoppy storage zone create --name <name> --region <region> --replicate-to <region>...
hoppy storage zone update --id <id> --custom-404 ...
hoppy storage zone delete --id <id>
```

The existing `upload/download/ls/rm` would move under `hoppy storage object`
(or stay at the top level for back-compat). Either way, a new `zone`
subcommand group is the cleanest path.

## API surface

Storage zone CRUD lives under `https://api.bunny.net/storagezone/...` in
`specs/core-platform.json` — the routes are already known; no API discovery
needed.
