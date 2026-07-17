---
type: adr
title: No --api-key flag; BUNNY_API_KEY env var only
status: accepted
date: 2026-03-18
deciders:
  - hoppy-maintainers
---

# No --api-key flag; BUNNY_API_KEY env var only

## Context and Problem Statement

How should hoppy accept the bunny.net API credential? A `--api-key` CLI flag is
convenient but exposes the secret to the process table and shell history.

## Considered Options

- Accept the key via a `--api-key` command-line flag
- Read the key only from the `BUNNY_API_KEY` environment variable

## Decision Outcome

Chosen option: environment variable only. Per clig.dev guidance, secrets must
not be passed as flags because they are visible in `ps` output and shell
history. hoppy reads `BUNNY_API_KEY` and exposes no flag alternative.
