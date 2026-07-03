---
title: Test Environment Needed on bunny.net
type: action-item
priority: high
status: open
created: 2026-03-18
---

# Test Environment Needed on bunny.net

A dedicated **test environment** (separate bunny.net account or sub-account) is required before running any destructive API operations (create, update, delete) from automated tests or Claude Code sessions.

## Current State

- The `BUNNY_API_KEY` in use belongs to a **production account** with real pull zones.
- Only **read-only** (GET) API calls are safe to run against it.
- Integration tests use `wiremock` mock server with recorded fixtures — no live API calls.

## What's Needed

- A bunny.net test/sandbox account (or a sub-user with restricted permissions)
- Dedicated API key for the test account, stored separately (e.g. `BUNNY_TEST_API_KEY`)
- Optionally: a CI-safe test pull zone that can be created/deleted freely

## Why

Without a test environment, we cannot safely:
- Test `create_pull_zone`, `update_pull_zone`, `delete_pull_zone` against the real API
- Record fixtures for write operations
- Run end-to-end integration tests in CI

## Related

- [[testing/test-plan-v0.1.0]] — test plan requiring this environment
- [[development-roadmap]] — project roadmap
