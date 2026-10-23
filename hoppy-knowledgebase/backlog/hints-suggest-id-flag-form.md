---
title: Hints still suggest the --id form after iter-86 made the id positional
type: backlog
date: 2026-10-23
status: completed
priority: low
origin: dogfooding-2026-10-23
---

# Hints say `hoppy pull-zone get --id 5871708`

iter-86 made the id positional for plain-id commands ([[backlog/id-arg-remaining-commands]]), but about a dozen `tip:` / error-message strings still suggested `--id`. Nothing checked hints against the CLI, so they drifted.

Fix: all hint command lines live in `crates/hoppy-cli/src/hint_cmds.rs` and a test parses each with clap. Commands that still need `--id` (shield, `dns zone scan results`) are pinned by a test that fails once they accept the positional form. Fixed in [[iterations/iteration-88-pull-zone-origin-and-id-hints]].
