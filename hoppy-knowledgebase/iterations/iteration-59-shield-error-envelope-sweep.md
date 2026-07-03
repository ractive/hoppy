---
title: Iter-59 — Shield error envelope sweep (event-logs 401)
type: iteration
date: 2026-06-01
tags:
  - iteration
  - shield
  - errors
  - dx
status: completed
branch: iter-59/shield-error-envelope-sweep
---

# Iter-59 — Shield error envelope sweep

## Why

`shield event-logs` swallows a rich error body (`errorResponse.message`,
`errorKey`) on 401s and prints only "Shield API returned status 401
Unauthorized". The user loses the actionable message
("You can only view the past 3 days (72 hours) of Event Logs.") and
the structured error key.

Iter-50 handled the top-level shape; this picks up the `errorResponse`-
wrapped sub-object that some Shield endpoints use.

See [[backlog/shield-event-logs-discards-error-body]].

## Scope

### 1. Audit Shield response shapes [2/2]

- [x] Sweep every Shield client error path for response bodies of
      the shape `{ "errorResponse": { "message", "errorKey",
      "statusCode" } }`. Record the list in the PR description.
- [x] For each found, confirm whether iter-50's top-level handling
      already covers it or it needs a separate branch.

### 2. Implement [2/2]

- [x] Teach the Shield error-mapping helper to recognise the
      `errorResponse`-wrapped envelope and surface
      `Shield API error <status> (<errorKey>): <message>`
      (matching iter-50's format).
- [x] Keep a graceful fallback when both envelopes are absent
      (404 with no body, etc.).

### 3. Tests [2/2]

- [x] E2E mock test: `shield event-logs` future date → renders
      `Shield API error 401 (invalid_datetime_window.event_logs):
      You can only view the past 3 days (72 hours) of Event Logs.`
- [x] Regression test: a 401 with no body still produces a graceful
      message.

## Out of scope

- Top-level Shield error envelope (already handled by iter-50).
- Non-Shield surfaces.

## Acceptance Criteria

- [x] `hoppy shield event-logs --id <z> --date 2099-01-01` prints
      the structured error with `errorKey` + `message`.
- [x] Any sibling Shield endpoint using the same envelope is also
      covered.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[backlog/shield-event-logs-discards-error-body]]
- [[iterations/iteration-50-shield-202-error-envelope]]
- [[dogfooding/session-2026-06-01-round2]]
