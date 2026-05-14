---
title: stream library statistics shows "Engagement Score -1" for empty libraries
type: backlog
date: 2026-05-10
status: completed
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
through. The API sentinel `-1` should *always* map to `N/A` in text/table
output — including the "empty library" case. Only an actual numeric `0`
returned by the API should render as `0`; never substitute `0` for the
sentinel.

JSON output should keep the raw `-1` (machine-readable), but the table
view shouldn't show it.
