---
title: pull-zone create --name has empty help description
type: backlog
date: 2026-06-01
status: resolved
priority: low
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - help
  - polish
---

# `pull-zone create --name` has no help text

```text
$ hoppy pull-zone create --help
...
      --name <NAME>
          
      --origin-url <ORIGIN_URL>
          HTTP/HTTPS origin URL the Pull Zone fetches from. Mutually exclusive with --storage-zone-id
...
```

Every other flag on this subcommand has a useful one-line
description; `--name` is the only one with an empty body, which
reads like a TODO.

## Fix

Add a one-liner explaining what the name becomes (it's surfaced in
URLs as `<name>.b-cdn.net`, must be globally unique, has charset
restrictions):

> Pull Zone name. Becomes the hostname `<name>.b-cdn.net` and must
> be globally unique across bunny.net. Lowercase letters, digits,
> and hyphens only.

Sweep the rest of the workspace for flags with empty `help =` to
catch any other missed entries.

## Acceptance

- `hoppy pull-zone create --help` shows a non-empty description
  for `--name`.
- No other flag across the workspace has an empty help string.
