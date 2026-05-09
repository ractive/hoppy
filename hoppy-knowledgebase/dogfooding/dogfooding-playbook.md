---
title: Dogfooding Playbook — safe real-API testing
type: docs
date: 2026-05-09
status: active
tags: [dogfooding, qa, bunny-api, safety]
---

# Dogfooding playbook

This playbook lets you run hoppy against a real bunny.net account without destroying anything that is already there. Follow it before/after every iteration that touches an API surface.

## Pre-flight: pick the account

Two accepted modes:

1. **Dedicated test account (preferred)** — register a separate bunny.net account, set a low billing alert, never use it for production. All resources created here can be wiped without consequence.
2. **Shared account (fallback)** — only acceptable if every resource you create is prefixed with `hoppy-test-` (see "Naming prefix" below) so cleanup is grep-able. Never run a destructive op without first listing what matches the prefix.

Authenticate once: `hoppy auth login` (writes to the standard config path; see `hoppy auth --help` for non-default paths).

## Naming prefix

Every resource hoppy creates during dogfooding **must** start with `hoppy-test-`. Examples:

- pull zone: `hoppy-test-cdn-2026-05-09`
- storage zone: `hoppy-test-storage-roundtrip`
- DNS zone: `hoppy-test.example.invalid` (use a domain you own and can throw away)
- container app: `hoppy-test-app-smoke`

Why: the cleanup script greps for `hoppy-test-` and refuses to touch anything that doesn't match. If you forget the prefix, cleanup will leak the resource — you'll have to remove it by hand from the dashboard.

## The safe loop

```
1. Read-only smoke test:    hoppy <noun> list                         # confirm auth + no surprises
2. Cleanup script first:    hoppy-knowledgebase/dogfooding/cleanup.sh # idempotent, scoped to prefix
3. Build:                   cargo build --release
4. Use:                     ./target/release/hoppy <command>          # always with hoppy-test- prefix
5. Note friction:           hoppy-knowledgebase/backlog/<short>.md    # one item per friction point
6. Cleanup after:           hoppy-knowledgebase/dogfooding/cleanup.sh
7. Verify in dashboard:     visit dash.bunny.net and confirm no orphans remain
```

## Destructive-op rules

- Every `delete` / `purge` / `update` / `disable` command is "destructive". Always pass `--dry-run` first if available, then re-run without it.
- Never run a destructive op without either an interactive confirmation (current default) or an explicit `--yes`/`-y` flag.
- If `--dry-run` is missing on a command and you'd want it during dogfooding, file a backlog item — it should be added.

## live-api feature

`cargo test --workspace --features live-api --quiet` runs E2E tests that hit the real API. They:

- read credentials from the standard config (or env vars — see `bunny-api-core` for the resolution order)
- create resources prefixed `hoppy-test-` and tear them down at the end
- are gated so plain `cargo test --workspace` never touches the network

If a `live-api` test fails halfway and leaks a resource, the cleanup script (next section) is your fallback.

## Cleanup script

`hoppy-knowledgebase/dogfooding/cleanup.sh` is the single tool that walks every API surface, lists resources matching `hoppy-test-`, and deletes them. It is **idempotent** — safe to run before AND after a session. It must:

- list-and-skip anything not matching the prefix (defence in depth — never delete an unprefixed resource even if requested)
- print what it is about to delete and require `--yes` to proceed (default is dry-run)
- exit non-zero if any deletion fails so CI can catch leaks

The script is intentionally a shell script (not Rust) so anyone can read and audit it. It's the only polyglot tool in the repo and lives only in the knowledgebase, not the build.

## Friction → backlog

Anything that surprises, frustrates, or trips you up during a session goes into `hoppy-knowledgebase/backlog/` as a separate file. Use the `backlog` type:

```yaml
---
title: <short title>
type: backlog
date: <today>
status: planned
priority: low|medium|high|critical
origin: dogfooding-2026-05-09
---
```

This is how iterations get fed. The dogfooding session is where the next iteration's plan is born.

## Related

- [[../decision-log]] — convention rules
- [[../iterations/iteration-23-hyalo-best-practices]]
- [[../iterations/iteration-25-publish]]
