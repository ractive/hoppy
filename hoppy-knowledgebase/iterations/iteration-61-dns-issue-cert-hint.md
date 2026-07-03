---
title: Iter-61 — DNS issue-cert error hint for undelegated zones
type: iteration
date: 2026-06-01
tags:
  - iteration
  - dns
  - errors
  - dx
status: completed
branch: iter-61/dns-issue-cert-hint
---

# Iter-61 — DNS issue-cert error hint for undelegated zones

## Why

`hoppy dns zone issue-cert` on an undelegated zone returns a
documented expected error, but the message is generic:

```text
Error: bunny.net API error 500 (): An error has occurred.
```

The `--help` text already warns that this is the failure mode, but
the runtime error gives no breadcrumb to "you need to delegate the
zone". Adding a hint converts a confusing dead-end into an actionable
next step.

See [[backlog/dns-issue-cert-error-translation]].

## Scope

### 1. Implement [2/2]

- [x] In the `issue-cert` command handler, on a 500 with no
      structured error key, append a hint to the error message:
      ```text
      hint: the zone must be delegated to bunny.net nameservers
            before a certificate can be issued. Set NS records to
            the values from `hoppy dns zone get --id <z>`.
      ```
- [x] Keep the original upstream message — append, don't replace.

### 2. Tests [2/2]

- [x] E2E mock test: 500 response on `issue-cert` produces the
      hint.
- [x] Regression test: 500 on a different command does NOT get the
      hint.

## Out of scope

- Active DNS resolver pre-flight (option 2 in the backlog item) —
  too much for the win.
- Translating other generic 500 errors elsewhere — case-by-case.

## Acceptance Criteria

- [x] `hoppy dns zone issue-cert` on an undelegated zone produces
      an error message that mentions delegation.
- [x] Successful issue-cert calls are unaffected.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[backlog/dns-issue-cert-error-translation]]
- [[dogfooding/session-2026-06-01-round2]]
