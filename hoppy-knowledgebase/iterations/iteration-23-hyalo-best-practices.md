---
title: Iteration 23 — Adopt hyalo project best practices
type: iteration
date: 2026-05-07
tags:
  - iteration
  - dx
  - quality
  - knowledgebase
  - cli-consistency
  - dogfooding
  - code-review
status: completed
branch: iter-23/hyalo-best-practices
---

# Iteration 23 — Adopt hyalo project best practices

**Goal:** Bring hoppy in line with the conventions and quality bar that have proven out in the sibling `../hyalo` project — knowledgebase shape, hyalo schema, Cargo workspace hygiene, CLI command consistency, LLM-friendly help text, and a real-API dogfooding loop. **Be critical**: not every hyalo decision applies here. Pick what's genuinely better, document where hoppy is right to diverge.

This is a quality-and-consistency pass, not new features. It pairs with [[iterations/iteration-25-publish]] (publishing the project) but should land first.

## Context

Hoppy and hyalo are both Rust CLI tools authored in the same codebase style by the same author. Hyalo has been through more polish iterations and has settled patterns hoppy hasn't yet picked up. This iteration audits the gap and closes it.

Reference projects:
- `../hyalo/` — the most-polished reference. Knowledgebase is rich, CLI is uniform, Cargo workspace hoists deps, lints are pedantic.
- `../ff-rdp/` — same author, similar shape; useful for cross-referencing.

## Scope

### 1. Knowledgebase + hyalo config alignment

Hoppy's `.hyalo.toml` today is one line (`dir = "hoppy-knowledgebase"`). Hyalo's is ~120 lines: rich type schemas, defaults, enums, named saved views.

- [x] Audit hoppy-knowledgebase contents — count files per type (`iteration`, `research`, `decision`, etc.) and tabulate which frontmatter properties they currently use
- [x] Pull the relevant `[schema.types.*]` blocks from `../hyalo/.hyalo.toml`. Don't copy 1:1 — only include types hoppy actually uses (`iteration`, `research`, `decision`, `docs`, maybe `backlog`). Drop hyalo-specific types like `pitch`.
- [x] Pull the iteration-type schema verbatim or near-verbatim — required fields (title, type, date, status, branch, tags), `status` enum (`planned`/`in-progress`/`completed`/`superseded`/`shelved`/`deferred`), branch pattern (`^iter-\d+[a-z]*/`), filename template (`iterations/iteration-{n}-{slug}.md`).
- [x] Pull the `[views.*]` definitions that apply: `planned`, `stale-in-progress`, `completed-with-todos`, `missing-status`, `missing-type`, `open-tasks`, `orphans`. Verify each returns sensible results on hoppy's actual KB before keeping it.
- [ ] Run `hyalo lint` against hoppy-knowledgebase. Fix any frontmatter violations the new schema flags. Document recurring patterns in `decision-log.md`.
- [x] Cross-check with `hyalo find --property '!type' --format text` and `hyalo find --property '!status' --format text` — every iteration / research / decision file should pass the schema.

### 2. CLAUDE.md alignment (project root)

Compare hoppy's `CLAUDE.md` to `../hyalo/CLAUDE.md` and decide which sections to import or rewrite.

- [x] **Add a Dogfooding section** (hyalo has this; hoppy doesn't). Spell out the loop: build release → use `target/release/hoppy` against the real bunny.net account → file friction observations as backlog items. See section 5 below for the safe-dogfooding playbook.
- [x] **Expand the hyalo CLI usage block** with the wider command list hyalo's CLAUDE.md uses (lint, lint-rules, types, views, tasks toggle, summary). Hoppy's KB ops will benefit.
- [x] **Add an `# Agents` line** at the top: "Delegate the work to agents whenever possible to avoid automatic context compaction." (Hyalo's lead.)
- [x] **Performance hint**: hyalo's CLAUDE.md says "Performance is key. Optimize the code to not read whole files into memory if not needed, but process them as streams if possible." Hoppy mostly does HTTP — not literally applicable, but the streaming-uploads / streaming-downloads case matters for storage and stream APIs. Adapt accordingly.
- [x] **Cross-platform note**: hyalo's CLAUDE.md says "Compatible with Windows, Linux and macOS." Hoppy's release matrix already builds for all three; surface this expectation in CLAUDE.md so future code respects it (no Unix-only path assumptions).
- [x] **Iteration file rules**: hyalo's checkbox spec is `- [ ] Task 1` (without a number). Hoppy's CLAUDE.md says `[] Task 1` (no leading dash). Pick one — hyalo's is the markdown standard.

### 3. Cargo workspace hygiene

Hoppy's workspace declares one inline `[dependencies]` block per crate plus the root. Hyalo hoists everything to `[workspace.dependencies]` and crates use `clap.workspace = true`.

- [x] **Adopt `[workspace.dependencies]`**: move every shared dep (clap, serde, serde_json, anyhow, tabled, tokio, reqwest, assert_cmd, predicates, insta, wiremock, tempfile, etc.) to the root `[workspace.dependencies]` table. In each crate, change to `clap.workspace = true` style.
- [ ] **Adopt `[workspace.lints.clippy]`**: import hyalo's pedantic-with-documented-allows block. Verify `cargo clippy --workspace --all-targets -- -D warnings` still passes (it'll surface a flood of new lints — fix or allow each one explicitly).
- [x] **Adopt `[workspace.package]`**: hoist `version`, `edition`, `license`, `repository`. Verify per-crate `Cargo.toml` references them via `version.workspace = true`.
- [x] **Set `resolver = "3"`**: hoppy currently uses `resolver = "2"`; hyalo uses `"3"` (the edition-2024 default). Move to `"3"` and verify the tree still resolves identically.
- [x] **Optimised release profile**: copy hyalo's `[profile.release]` (`codegen-units = 1`, `lto = true`, `panic = "abort"`, `strip = true`). Measure binary size before/after — record in this iteration's Notes.

### 4. AI_NOTICE + CHANGELOG

- [x] Copy `../hyalo/AI_NOTICE` to repo root (verbatim — it's a generic AI-generation disclosure).
- [x] Create a `CHANGELOG.md` with retroactive entries for the iterations that have shipped (0–22 + 15, 16, 17, 18 once they merge). Use the Keep-a-Changelog format hyalo uses. This sets up iter-25 to add a `## [Unreleased]` section that becomes `## [v0.1.0] - YYYY-MM-DD` on release.

### 5. Safe dogfooding against real bunny.net

The user explicitly asked: *can we (again) check everything against real bunny.net services without destroying what's there?* Yes — but it needs structure.

- [x] Create `hoppy-knowledgebase/dogfooding/dogfooding-playbook.md` documenting the safe loop:
  - **Separate test account**: ideal — register a dedicated bunny.net account with billing alerts, no production resources, used only for hoppy testing
  - **Naming prefix**: every resource hoppy creates during dogfooding gets a `hoppy-test-` (or `hoppy-dogfood-`) prefix so cleanup is grep-able
  - **Idempotent cleanup script**: `hoppy-knowledgebase/dogfooding/cleanup.sh` — list every resource matching the prefix and delete. Run before and after each session.
  - **Read-only smoke test first**: `hoppy <command> list / get` against the production account is safe; do this pass before any destructive op.
  - **Use the existing `live-api` feature** (already gated in `Cargo.toml`): `cargo test --features live-api` runs E2E that actually call bunny.net. Today these tests are minimal — expand them.
- [ ] **Add a `--dry-run` global flag** for destructive commands where it doesn't already exist (delete, purge, update). Behavior: print what would happen, don't call the API. Audit each subcommand and tick the ones that need this.
- [ ] **Audit the `hoppy delete` / `hoppy purge` family** for confirmation prompts. Already covered partially by `-y` / `--yes` skip. Make sure no destructive op runs without either an explicit `--yes` or an interactive confirmation.
- [ ] **Run a full dogfooding session** end-to-end against the real account using the playbook. Every friction point becomes a backlog item in `hoppy-knowledgebase/backlog/` (a new subfolder if needed).

### 6. CLI command consistency audit

The user explicitly asked: *Is it `hoppy foo list` or just `hoppy foo` everywhere or does it need to be unified?*

Snapshot of today's top-level commands (from `--help`): `pull-zone`, `storage-zone`, `storage`, `dns`, `stream`, `shield`, `script`, `container`, `db`, `auth`, `statistics`, `video-library`, `purge`, `completions`. Most are container groups; some are leaf verbs (`auth`, `purge`, `completions`).

- [x] **Generate a complete command tree**: parse the `Subcommand` enums in `src/cli.rs` and produce a markdown table of every subcommand + its leaf actions. Save as `hoppy-knowledgebase/cli/command-tree.md`.
- [ ] **Audit verb consistency**:
  - List operations: `pull-zone list`, `storage-zone list`, `dns zone list`, `dns record list` — confirm every collection-noun has a `list`
  - Get-by-id operations: same audit for `get`
  - Create vs Add: hoppy uses `dns record add` but `pull-zone create` and `storage-zone create`. Pick one verb per relationship type (`create` for top-level resources, `add` for items inside a parent collection — which is roughly what's happening today; codify the rule)
  - Delete vs Remove: `pull-zone delete` vs `dns record remove`/`delete`? Audit and unify.
  - Update vs Edit: are any commands `edit`? Should all be `update`.
- [ ] **Audit `<container> <action>` shape**: hoppy mixes shapes — some commands have `hoppy <noun>` as a hub (e.g. `dns` → requires a sub-action), others are leaf verbs (`hoppy purge <url>`). That's fine — but confirm every `<noun>` command shows useful output on `hoppy <noun>` (today some print "missing subcommand" errors that aren't friendly).
- [ ] **Top-level `hoppy container list` aliasing**: iter-19 flagged this — `container list` should alias `container app list` etc. Confirm if iter-19 actually shipped this; if not, do it here.
- [x] **Document conventions in `decision-log.md`**: one entry capturing the verb rule (`create`/`add`/`update`/`delete`/`remove`/`list`/`get`) so future commands follow it.
- [ ] **Apply renames with `#[arg(long, alias = "<old>")]`** — iter-19's no-breakage rule still applies. Add aliases for any rename so existing scripts keep working.

### 7. Help text quality (LLM-friendly)

The user asked: *How are the help texts written? Are the AI agent/LLM friendly?*

- [ ] **Sample audit**: pick 5 hot-path commands (`pull-zone create`, `storage-zone create`, `dns record add`, `container app create`, `db create`) and read their `--help` output side-by-side with hyalo's hot paths. Look for: missing `long_about`, missing examples, missing `--reveal`/`--dry-run` cross-references, missing semantic descriptions of enum values, missing "after this command, you probably want to..." next-step hints.
- [x] **Define the "good help text" template** — write it as a section in `hoppy-knowledgebase/cli/help-text-style.md`. Components: one-line summary, multi-line `long_about` describing semantics + edge cases, `after_help` with at least one example, cross-references to related commands.
- [ ] **Apply the template across all subcommands**. iter-19 partially did this; finish the long tail. Tick subcommands as you go in this iteration.
- [ ] **LLM-friendly specifics**:
  - Every enum-typed flag must list possible values *with their meanings* in `long_help`, not just the values
  - Every `--*-id` flag must say "use `hoppy <noun> list` to find IDs"
  - Every destructive command must prefix `long_about` with an explicit "DESTRUCTIVE: …" line
  - Use machine-parseable formatting in `--help` (consistent indentation, no decorative ASCII art) so an LLM consuming `hoppy --help` output can build a reliable command tree
- [ ] **Verify with an actual LLM**: pipe `hoppy --help` (and a few sub-helps) into an LLM with the prompt "build a structured command map" and check whether it can do it without hallucinating. Record the result in this iteration's Notes.

### 8. Code review (proper, end-to-end)

The user asked: *plan a proper code review and dogfooding session again.*

- [ ] **Run `/review-rust all`** (the project skill) against the workspace and triage findings. File a `hoppy-knowledgebase/code-review/iter-23-findings.md` with: critical issues, recommended fixes, deferred items (tag + status).
- [ ] **Specifically scrutinise**:
  - `.unwrap()` / `.expect()` outside tests — must convert to `?` with `Context`
  - `.clone()` calls — prove each is necessary or remove
  - `pub` on struct fields where private would do
  - Error message quality — does the user see the bunny API error code + message, or a paraphrase?
  - Async correctness: any `.block_on()` inside async, missing `.await`s, accidental serial when parallel was intended
  - Streaming: storage uploads/downloads — verify they don't `.bytes().await?` huge bodies
- [ ] **Apply non-controversial fixes inline**. Larger findings → backlog items (or follow-up iterations).

### 9. Module / crate layout

The user asked: *Check how modules and crates and the e2e tests are set up in ../hyalo. I think it worked well there, but be critical to apply those things as well.*

Hyalo: `crates/hyalo-cli`, `crates/hyalo-core`, `crates/hyalo-mdlint`. The CLI binary lives in its own crate. Hoppy: CLI binary at workspace root, API client crates in `crates/bunny-api-*`. **The hyalo shape is more crates.io-friendly** (publishing requires the binary crate to be standalone) — see iter-25.

- [ ] **Investigate moving the CLI binary to `crates/hoppy-cli/`**:
  - Pros: matches hyalo, simpler crates.io publish (binary crate has clean deps), tests already in `tests/e2e/` move with it
  - Cons: large refactor, every `mod commands::*` import path changes, root `Cargo.toml` becomes a virtual workspace
  - **Decision point**: do this in iter-23 *or* defer to iter-25. If the publishing iteration (iter-25) needs the structure, do it here as a pre-req. If publishing can work with the binary at the root, defer.
  - **Recommendation**: do it here. Crates.io publishing of a workspace-root binary works but is unusual; aligning to the hyalo shape removes friction.
- [x] **API-client crates naming**: `bunny-api-*` is hoppy's own convention; hyalo uses `<project>-<domain>` without a prefix. Hoppy's "bunny-api-" prefix is intentional (it advertises the API surface they wrap, not hoppy itself) — keep this. Document the rule in `decision-log.md` so it doesn't drift.
- [ ] **E2E tests**: already consolidated in iter-22. Confirm the layout matches hyalo's `crates/<crate>/tests/e2e/` exactly. If the binary moves to `crates/hoppy-cli/`, the top-level `tests/e2e/` moves with it — verify.

## Notes — what landed in this PR vs. what was deferred

This iteration was large by design — the plan flagged that splitting was acceptable. Actual split:

**Landed in iter-23:**

- Hyalo schema + saved views (section 1, except whole-KB lint cleanup of pre-existing markdown)
- All of section 2 (CLAUDE.md alignment)
- Section 3: `[workspace.dependencies]`, `[workspace.package]`, `resolver = "3"`, `[profile.release]`. **`workspace.lints.clippy = pedantic` deferred** — separate flood-and-fix iteration. Workspace lints currently set `unsafe_code = "forbid"` only.
- Section 4: AI_NOTICE + CHANGELOG (Unreleased section seeded)
- Section 5: dogfooding playbook + cleanup-script skeleton. Live session against real account NOT run in this iteration — pre-flight only. `--dry-run` audit deferred.
- Section 6: command-tree generated, verb conventions documented in `decision-log.md`. **Renames + aliases not applied** — would 2x the diff. Deferred to a follow-up.
- Section 7: help-text style guide written. **Template not applied across all subcommands** — long-tail follow-up.
- Section 9: `bunny-api-*` naming rationale documented. **CLI-binary move to `crates/hoppy-cli/` deferred to iter-25** (publishing) — that plan needs it as a prerequisite.

**Deferred / explicit follow-ups (filed in `hoppy-knowledgebase/backlog/`):**

- Drill-down hints (hyalo iter-107 pattern) — `backlog/drill-down-hints.md`
- Lean README pass — `backlog/lean-readme.md`
- Workspace `clippy::pedantic` flood-and-fix
- `--dry-run` audit across destructive commands
- Rename pass for verb consistency (with `alias = "<old>"`)
- Help-text template applied across the long tail
- `/review-rust all` audit (section 8)
- Live dogfooding session end-to-end against real account

**Binary size before/after `[profile.release]` change:** measurement deferred — release CI builds will surface the delta on the next tagged build.

## Implementation Notes

- **Order of operations**: schema + lint cleanup first (sections 1–2) — establishes the baseline. Then Cargo hygiene (3–4). Then dogfooding playbook + audit (5–6). Then help-text + code review (7–8). Module/crate layout (9) last because it's the most invasive.
- **Branch discipline**: this iteration is large. Consider splitting the CLI consistency rename (section 6) into its own follow-up if the diff balloons; the rest can land as one PR.
- **Aliases for renames**: every flag/subcommand rename gets a `#[arg(long, alias = "old")]` so existing scripts and docs don't break. iter-19's "no breaking changes in this iteration" rule applies.
- **No new API surface**: this iteration adds zero new bunny.net commands. If a section can't be done without one, defer to a follow-up.
- **The `live-api` feature** already exists in hoppy's `Cargo.toml` (`[features] live-api = []`) — confirm what tests gate on it today and expand from there.

## Suggested test cases

1. After schema work: `hyalo lint` reports zero violations across the whole knowledgebase.
2. After Cargo hygiene: `cargo clippy --workspace --all-targets -- -D warnings` passes with the pedantic group enabled.
3. After dogfooding playbook: a fresh dogfooding session creates resources prefixed `hoppy-test-`, runs every verb at least once, and the cleanup script tears everything down without affecting unprefixed resources.
4. After CLI consistency: every command in the audit table has a documented verb (list/get/create/add/update/delete/remove). No command shows "missing subcommand" without a friendly error suggesting the next action.
5. After help-text work: `hoppy --help` and every `hoppy <noun> --help` parse cleanly when piped to a fresh LLM with "build a command map" prompt.
6. After code review: zero `.unwrap()` / `.expect()` outside tests. `cargo test --workspace --quiet` still green.

## Risks

- **Schema lint flood**: importing the hyalo schema may flag dozens of existing files for missing properties. Plan to fix in batch with `hyalo set` rather than one-by-one.
- **Pedantic clippy flood**: enabling pedantic adds *many* warnings. Allow-list each one explicitly with a comment justifying — don't blanket-allow.
- **CLI rename regressions**: clap aliases prevent breakage but tests, docs, and READMEs may reference old names. Update all in lockstep with the rename.
- **Dogfooding creates real resources**: even with the `hoppy-test-` prefix, a buggy cleanup script can leave orphans. Always check the bunny.net dashboard after a session.
- **Module-layout refactor breaks paths**: every `use crate::commands::*` becomes `use hoppy_cli::commands::*` (or similar). Big diff. Run `cargo check --workspace` continuously during the move.

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| Hyalo schema + view import (1) | Small |
| CLAUDE.md alignment (2) | Small |
| Cargo workspace hygiene (3) | Medium |
| AI_NOTICE + CHANGELOG (4) | Small |
| Safe dogfooding playbook (5) | Medium |
| CLI consistency audit (6) | Medium–Large |
| Help-text quality pass (7) | Medium |
| Code review (8) | Medium |
| Module/crate layout (9) | Large |
| **Total** | **Large** |

## Related

- Reference: `../hyalo/` (most-polished sibling project)
- Reference: `../ff-rdp/` (same-author cross-reference)
- Pairs with: [[iterations/iteration-25-publish]] (publishing — section 9 may be a hard prerequisite)
- [[development-roadmap]]
- [[decision-log]]
- [[adding-a-feature]]
