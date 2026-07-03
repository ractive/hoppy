---
title: --debug shows request URL + response body but not the request body
type: backlog
date: 2026-05-26
status: resolved
priority: low
origin: dogfooding-2026-05-26 (post-iter-38)
---

# `--debug` is missing the request body

Today's `--debug` output:

```
>> POST https://api.bunny.net/pullzone/5857625
<< 200 OK
<<< {"Id":5857625,"Name":"...","LogForwardingHostname":"old",...}
```

The request body — the most useful piece for diagnosing "why did my update
not stick?" — is omitted. When investigating the
[[log-forwarding-hostname-silent-noop]] bug, it was impossible to tell
from `--debug` whether the CLI sent the hostname or not.

Add `>>>` lines for the request body (with secret redaction matching the
existing `--reveal` semantics).

## Resolution

Fixed in [[iterations/iteration-39-dogfooding-2026-05-26-fixes]].
