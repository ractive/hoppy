---
title: "`pull-zone get` table is 11 columns wide — unreadable on narrow terminals"
type: backlog
date: 2026-05-26
status: planned
priority: low
origin: dogfooding-2026-05-26 (post-iter-38)
---

# `pull-zone get` table is too wide

`hoppy pull-zone get --id <id>` (table format) renders 11 columns side by
side:

```
| ID | Name | Origin URL | CNAME | Type | Enabled | Suspended | Bandwidth Used | Bandwidth Limit | Hostnames | ... |
```

On a typical 120-column terminal, this wraps unreadably. Single-resource
"get" output should be vertical (Field / Value), like `auth check`:

```
+--------------+-------------------+
| Field        | Value             |
+--------------+-------------------+
| ID           | 5857625           |
| Name         | hoppy-test-...    |
| Origin URL   | https://...       |
| Enabled      | true              |
+--------------+-------------------+
```

`list` is the right shape for the wide table; `get` should pivot.

The same fix likely applies to other single-resource gets that currently
use the horizontal layout.
