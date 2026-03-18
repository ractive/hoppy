---
description: >
  Perform a critical Rust code review. Pass an argument to specify scope:
  a branch name for PR review, a crate/file path for focused review,
  or "all" for full codebase review. Examples: /review-rust iter-1/api-clients,
  /review-rust crates/bunny-api-compute, /review-rust all
allowed-tools: Read, Glob, Grep, Bash, LSP, Edit, Write, Agent
---

Read the skill file at .claude/skills/review-rust/SKILL.md and follow its instructions to perform a code review.

The user's review request: $ARGUMENTS
