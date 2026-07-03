---
title: "Iteration 7 — Code Cleanup & Small Features"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - cleanup
  - tech-debt
  - polish
status: completed
branch: iter-7/cleanup
---

# Iteration 7 — Code Cleanup & Small Features

**Goal:** Quick wins — clean up deferred tech debt, add small features. No release infrastructure yet.

- [x] `hoppy auth check` — validate API key and print billing/account info (`GET /billing` endpoint + CLI command + 3 wiremock tests)
- [x] Replace `endpoint_suggestions: Vec<serde_json::Value>` in `bunny-api-containers` types with concrete `EndpointSuggestion` struct
- [x] Remove `CursorListJson` wrapper in CLI — serialize `CursorList<T>` directly in JSON mode (5 call sites updated)
- [x] Add `FromStr` impls to container enums (`RuntimeType`, `Granularity`, `RegistryType`, `LogForwardingType`, `SyslogFormat`) — replaced 5 hand-written `parse_*` helpers + 5 unit tests
- [x] Wire WAF profiles CLI command (`shield waf profiles`)
- [x] Wire Shield zone lookup by pull zone (`shield zone get-by-pullzone`) — already existed from iter 5
- [x] Wire container autoscaling commands (`container app autoscaling-get|autoscaling-update`)
- [x] Wire container region settings commands (`container app region-settings-get|region-settings-update`)
- [x] Wire container registry image commands (`container registry image-tags|image-digest|config-suggestions|search-public`)
- [x] Wire compute upsert commands (`script variable upsert`, `script secret upsert`)
- [x] Progress bars for storage upload and video upload (`indicatif` crate, streaming uploads with determinate bar, stderr only when TTY, suppressed by `--quiet`); storage download uses an indeterminate spinner (client buffers the full response before writing)
- [x] `bunny-api-containers` wiremock integration tests — already had 57 tests covering all endpoints (confirmed, no gaps)

**Deliverable:** Cleaner codebase, all deferred small items resolved.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-6-scripting-containers]] — previous iteration
- [[iterations/iteration-1-code-review]] — code review that identified some of these items
