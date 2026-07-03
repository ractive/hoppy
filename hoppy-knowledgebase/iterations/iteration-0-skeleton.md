---
title: "Iteration 0 — Project Skeleton"
type: iteration
date: 2026-03-17
tags:
  - iteration
  - skeleton
  - foundation
status: completed
branch: iter-0/skeleton
---

# Iteration 0 — Project Skeleton

**Goal:** Rust project compiles, CLI parses args, nothing talks to the network yet.

- [x] `cargo init` with workspace layout
- [x] Clap derive setup with nested subcommand structure (`hoppy <service> <action>`)
- [x] Global flags: `--format json|table|text`, `--debug`, `--quiet`, `--yes`, `--version`
- [x] `BUNNY_API_KEY` env var reading (validate presence, error if missing)
- [x] Output formatting module (json + table + text) with a dummy data struct
- [x] Error handling scaffold (anyhow, human-friendly error display, JSON errors when `--format json`)
- [x] Stderr for status/errors, stdout for data
- [x] `hoppy completions <shell>` subcommand (clap_complete)
- [x] Basic README with usage examples
- [x] CI: GitHub Actions workflow that builds and runs `hoppy --help` on linux/mac/windows

**Deliverable:** `hoppy --help` shows the command tree, `hoppy pull-zone list` prints "not implemented yet" cleanly.

## Related

- [[development-roadmap]] — project roadmap
- [[Seed]] — original project brief
- [[research/rust-cli-best-practices]] — CLI best practices research
- [[research/cli-design-patterns]] — CLI design patterns
