---
title: container app create --min -1 fails with "unexpected argument"
type: backlog
date: 2026-06-01
status: planned
priority: low
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - container
  - clap
  - dx
---

# `container app create --min -1` is rejected as "unexpected argument"

```sh
$ hoppy container app create --name probe --runtime-type Shared --min -1 --max 1
error: unexpected argument '-1' found

Usage: hoppy container app create [OPTIONS] --name <NAME> --runtime-type <RUNTIME_TYPE> --min <MIN> --max <MAX>
```

This is the classic clap footgun: `-1` is parsed as a short flag,
not a value for `--min`. The error is confusing — the user thinks
"-1" is an unknown option, when really clap can't tell value from
flag.

Two reasonable fixes:

1. **`allow_hyphen_values = true`** on `--min`/`--max` (and any
   sibling numeric flags that could plausibly be negative). Then
   validate `n >= 0` in code with a clear "min must be >= 0" error.
2. **Document `--min=-1`** (the `=` form sidesteps the parser) in
   the flag's help text — cheap but not as friendly.

Sweep for the pattern across all `--min`/`--max`/`--ttl`/`--priority`/
`--weight`/`--port` flags so the policy is uniform.

## Acceptance

- `hoppy container app create --min -1 ...` produces a domain
  validation error (`min must be >= 0`) rather than a clap parse
  error.
- All numeric flags across the workspace either accept negative
  values cleanly or surface a clap-aware error explaining the
  workaround.
