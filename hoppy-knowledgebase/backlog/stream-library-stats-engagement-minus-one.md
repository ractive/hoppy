---
title: stream library statistics shows "Engagement Score -1" for empty libraries
type: backlog
date: 2026-05-10
status: planned
priority: low
origin: dogfooding-2026-05-10
---

# Sentinel `-1` leaks to the user

`hoppy stream library statistics --library-id <id>` on a brand-new library
prints:

```
Engagement Score    -1
```

`-1` is bunny.net's "no data" sentinel. Hoppy's text formatter passes it
through. Replace with `N/A` (or `0` if the score is genuinely zero when
nothing has played).

Possibly the same in JSON output — there it's arguably fine to keep `-1`
as it's machine-readable, but the table view shouldn't show it.
