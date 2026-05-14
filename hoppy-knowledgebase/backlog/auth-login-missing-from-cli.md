---
title: dogfooding playbook references `hoppy auth login` but only `auth check` exists
type: backlog
date: 2026-05-10
status: completed
priority: high
origin: dogfooding-2026-05-10
---

# Doc/CLI mismatch on `hoppy auth …`

`hoppy-knowledgebase/dogfooding/dogfooding-playbook.md` (Pre-flight section)
tells the reader to run `hoppy auth login` before dogfooding. That subcommand
does not exist — `hoppy auth --help` lists only `check` (and `help`).

Two ways to resolve:

1. **Fix the docs** — replace `hoppy auth login` with the actual flow:
   set `BUNNY_API_KEY` (env var) and run `hoppy auth check` to confirm it
   works. The top-level `--help` already says
   "Set the BUNNY_API_KEY environment variable to authenticate."
2. **Add a real `auth login`** — interactive prompt that writes the key to
   the standard config path. The playbook implies this already exists.

The "writes to the standard config path; see `hoppy auth --help` for non-default
paths" hint in the playbook is also stale — `auth --help` only mentions
the global `--reveal`, etc., not any config path.

Recommended: drop the references to `auth login` and config paths in the
playbook for now and document that auth = `BUNNY_API_KEY` env var.
