---
title: dns issue-cert undelegated returns generic 500 ()
type: backlog
date: 2026-06-01
status: resolved
priority: low
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - dns
  - errors
  - dx
---

# `dns issue-cert` on undelegated zone returns generic 500

```sh
$ hoppy dns zone issue-cert --id <fresh-zone>
Error: bunny.net API error 500 (): An error has occurred.
```

The command's own `--help` text explicitly calls this out:

> The zone must be properly delegated to bunny.net nameservers — the
> certificate authority needs to validate the domain via DNS challenge.
> If the zone isn't delegated, the API returns an error.

So this is a *documented expected case*, but the 500 with empty error
key and the generic message is unhelpful. The user has no breadcrumb
back to "this means I need to delegate".

## Fix options

1. **Translate the 500** — when issue-cert returns a 500 with no
   structured error, append a hint:
   ```
   Error: bunny.net API error 500: An error has occurred.
     hint: the zone must be delegated to bunny.net nameservers
           before a certificate can be issued. Set NS records to
           the values from `hoppy dns zone get --id <z>`.
   ```
2. **Pre-flight the delegation** — before calling issue-cert,
   resolve the zone's NS records and compare to bunny.net's. If
   they don't match, fail fast with a clear message and skip the
   API call.

Option 1 is cheaper and good-enough. Option 2 is nicer but needs
a DNS resolver and timeout-handling. Start with option 1.

## Acceptance

- Running `issue-cert` on an undelegated zone produces an error
  message that mentions delegation.
- Other 500 paths (genuine upstream issues) are unaffected — the
  hint is appended, not replacing the upstream message.
