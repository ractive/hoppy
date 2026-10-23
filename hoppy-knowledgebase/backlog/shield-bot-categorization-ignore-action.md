---
title: bot-categorization list fails on new action 3 = Ignore
type: backlog
date: 2026-10-23
status: completed
priority: high
origin: openapi-drift-2026-10-23
tags:
  - shield
  - bot-categorization
  - api-drift
  - deserialization
resolved-in: iter-87
---

# `BotCategorizationAction` gained `3 = Ignore`

**Impact: breaking.** `BotCategorizationDetails.action`
(`crates/bunny-net-api/src/shield/types.rs:2055`) is a non-optional closed
repr enum (`None/Block/Allow`), so `shield bot-categorization list` fails
once any bot is set to Ignore.

The spec now splits the type:

- per-bot (`BotCategorizationDetails.action`, `UpdateBotCategorizationConfigurationRequest`): `BotCategorizationAction` 0–3 (`None, Block, Allow, Ignore`)
- category-level (`BotCategorizationGroup.categoryAction`, `BotCategoryDetails.action`, and the `PUT …/categories/{category}` body, now `UpdateBotCategoryConfigurationRequest`): new `BotCategoryAction` 0–2

- [x] Add `Ignore = 3` for per-bot actions and a separate category-action type without it
- [x] Add a forward-compatible fallback on response fields
- [x] Expose `ignore` in the per-bot CLI command only

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

`BotCategorizationAction` gained `Ignore = 3`; new `BotCategoryAction` (0-2) for category actions, with its own request body. `set-bot --action 3` works; `set-category --action 3` exits 2 via a clap range check before any request.

See [[iterations/iteration-87-openapi-drift-fixes]].
