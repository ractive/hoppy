---
title: shield metrics waf-rule uses --shield-zone-id while siblings use --id
type: backlog
date: 2026-06-01
status: planned
priority: medium
origin: dogfooding-2026-06-01
tags:
  - cli
  - shield
  - dx
  - consistency
---

# `shield metrics waf-rule` has the wrong shield-zone flag

Every other `shield metrics <sub>` subcommand uses `--id` for the
Shield Zone ID:

```sh
hoppy shield metrics overview        --id <z>
hoppy shield metrics detailed        --id <z>
hoppy shield metrics rate-limits     --id <z>
hoppy shield metrics bot-detection   --id <z>
hoppy shield metrics upload-scanning --id <z>
hoppy shield metrics rate-limit      --id <r>   # this --id is rule ID, not zone
```

But `shield metrics waf-rule` uses `--shield-zone-id`:

```sh
hoppy shield metrics waf-rule --id 118829 --rule-id 1
# error: unexpected argument '--id' found
# tip: a similar argument exists: '--shield-zone-id'

```

This is the only odd one out. Two reasonable fixes:

1. **Promote consistency**: rename `--shield-zone-id` to `--id` (with
   `--shield-zone-id` as a hidden alias for back-compat).
2. **Promote clarity**: rename `--id` to `--shield-zone-id` on *all*
   the metrics subcommands and keep `--id` as a hidden alias.

Option 1 matches the existing siblings; option 2 is more self-documenting.
Pick one and apply across `shield metrics *`.

## Reproduction

```sh
hoppy shield metrics waf-rule --id 118829 --rule-id 1   # ❌
hoppy shield metrics waf-rule --shield-zone-id 118829 --rule-id 1  # ✅
```

## Acceptance

- `hoppy shield metrics waf-rule --id <z> --rule-id <r>` works.
- All `shield metrics *` subcommands use the same flag name for Shield
  Zone ID.
