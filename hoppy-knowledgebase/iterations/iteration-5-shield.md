---
title: "Iteration 5 — Shield (Security)"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - shield
  - security
  - waf
status: completed
branch: iter-5/shield
---

# Iteration 5 — Shield (Security)

**Goal:** Manage WAF, rate limiting, DDoS settings.

- [x] Shield zone commands:
  - [x] `shield zone list|get|get-by-pullzone|create|update`
- [x] Shield subcommands:
  - [x] `shield waf list-rules|get-rule|add-rule|update-rule|delete-rule`
  - [x] `shield rate-limit list|get|create|update|delete`
  - [x] `shield access-list list|get|create|update|delete|update-config`
  - [x] `shield bot-detection get|update`
- [x] DDoS configuration via shield zone update (--ddos-sensitivity, --ddos-execution-mode, --ddos-challenge-window)
- [x] Debug mode support (`--debug` flag)
- [x] Confirmation prompts for destructive operations (`--yes` to skip)
- [x] 27 wiremock integration tests with fixture-based responses
- [x] Error handling tests (401 unauthorized, 404 not found)
- [x] WAF profiles command (`shield waf profiles`) — wired in iter 7

**Deliverable:** Security configuration via CLI.

## Related
- [[development-roadmap]] — project roadmap
- [[iterations/iteration-4-stream]] — previous iteration
- [[api/bunny-api-quirks]] — Shield uses camelCase unlike Core API
- [[decision-log]] — DDoS as shield zone update, Shield enum values as integers
