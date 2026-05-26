---
title: Restructure README as lean landing page
type: backlog
date: 2026-05-09
status: resolved
priority: low
origin: iter-23 inspiration scan of ../hyalo git log
---

# Lean README

Hyalo recently restructured its README "as a lean landing page instead of an exhaustive manual" (commit `4b6df49`). The current hoppy README is exhaustive — useful as reference, but slow for first-time readers to scan.

## What lean looks like

- Hero: one paragraph + one example
- Install: 2–3 short blocks (`brew`, `cargo`, deb/rpm)
- Quick start: 3–5 commands that produce visible value (`auth login`, `pull-zone list`, `pull-zone create`)
- Link to `hoppy-knowledgebase/cli/command-tree.md` for the full surface
- Link to dash.bunny.net for concept docs

The exhaustive content can move to `hoppy-knowledgebase/` (where humans + LLMs already look) or to a separate `docs/MANUAL.md`.

Defer this until iter-25 publish work — the README is a first-impression artefact and shouldn't churn during normal feature work.
