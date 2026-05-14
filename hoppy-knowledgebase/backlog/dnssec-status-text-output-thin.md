---
title: dns zone dnssec status text output omits the DS record
type: backlog
date: 2026-05-10
status: completed
priority: low
origin: dogfooding-2026-05-10
---

# `dns zone dnssec status` is too terse in `--format text`

```sh
hoppy dns zone dnssec status --id <id> --format text
# id      790525
# domain  hoppy-test-1778429024.net
# enabled true
```

JSON has the full payload (DsRecord, Digest, KeyTag, Algorithm,
DsConfigured, …). The text/table view is the friendly one — and the most
important fact for a user wiring up DNSSEC at their registrar is the DS
record, which isn't shown.

Show the DS record line and the digest in text/table mode when DNSSEC is
enabled.

Iter-17 follow-up.
