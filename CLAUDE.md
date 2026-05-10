# Agents
Delegate the work to agents whenever possible to avoid automatic context compaction.

# Documentation

Keep all documentation in `./hoppy-knowledgebase/` as `*.md` markdown files with YAML frontmatter (text, numbers, checkboxes, dates, lists). Use it as your second brain:
- Research outcomes → `research/`
- Design decisions → `decision-log.md`
- Iteration plans → `iterations/iteration-NN-slug.md` (one file per iteration, markdown task lists for steps/tasks/ACs)
- CLI surface notes / command tree → `cli/`
- Dogfooding playbook → `dogfooding/`
- Backlog items (deferred friction / follow-ups) → `backlog/`

Organize in subfolders. Use `[[wikilinks]]` for cross-references. Keep Obsidian-compatible.

**Always use hyalo for knowledgebase interactions — never use Edit/Read/Grep directly:**
- **Search/filter**: `hyalo find --property status=planned --tag iteration`
- **Body search**: `hyalo find "broken links"` or regex: `hyalo find -e 'TODO|FIXME'`
- **Title regex**: `hyalo find --property 'title~=link'`
- **Overview**: `hyalo summary`, `hyalo properties`, `hyalo tags`
- **Mutate frontmatter**: `hyalo set`, `hyalo remove`, `hyalo append` (e.g., `hyalo set iterations/iteration-16-... --property status=completed`)
- **Toggle tasks**: `hyalo task toggle <path> --all`, `--section "Tasks"`, `--line 5,7,9`
- **Lint frontmatter + markdown body**: `hyalo lint`, `hyalo lint --rule MD013 --detailed`, `hyalo lint --rule-prefix HYALO`, `hyalo lint --strict`, `hyalo lint --fix --dry-run`, `hyalo lint --fix`, `hyalo lint --fix-rule HYALO001`
- **Manage lint rules**: `hyalo lint-rules list`, `hyalo lint-rules show MD013`, `hyalo lint-rules set MD013 --enabled false`, `hyalo lint-rules set MD013 --severity error`
- **Manage schemas**: `hyalo types list`, `hyalo types show <name>`, `hyalo types set <name> --required title,date`
- **Saved views**: `hyalo views`, `hyalo find --view planned`, `hyalo find --view stale-in-progress`
- Only fall back to Edit for body content changes (markdown prose) that hyalo can't handle
- **Do NOT pass `--dir hoppy-knowledgebase/`** — `.hyalo.toml` already sets it as the default
- **Use `--format text`** for compact LLM-friendly output

**Iteration file rules:**
- Always name `iteration-NN-slug.md` — no standalone plan files
- Frontmatter must include: `title`, `type: iteration`, `date`, `tags`, `status`, `branch`
- Status lifecycle: `planned` → `in-progress` → `completed` → `superseded`
- Add tasks as markdown checkboxes `- [ ] Task 1` (markdown standard, with leading dash and space)
- Mark tasks as completed only after verifying that they were done

# Rust

## Language Server
Use the rust-analyzer LSP plugin for code intelligence: analyzing code, finding references, go-to-definition, checking clippy warnings.
Run "cargo check" before using it to update its indexes, after changing *.rs files.

## Code Quality Gates
Make the code unit testable. Add tests if feasible. Add e2e tests for all commands/subcommands.

Performance is key. For storage uploads/downloads and stream APIs, prefer streaming bodies (`reqwest::Body::wrap_stream`, `bytes_stream()`) over reading whole payloads into memory. Apply this anywhere a single request can carry an arbitrarily large blob.

It must be compatible with Windows, Linux and macOS — no Unix-only path assumptions, no shell-specific commands in code paths, use `std::path::PathBuf` and forward-slash-tolerant joining.

Before committing or creating a PR, run **in this order** and fix all issues:
1. `cargo fmt`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --quiet`

Never skip a step. Never commit code that fails any of these.
Do *not* merge with `--squash`.

## Code Patterns
- No `.unwrap()` / `.expect()` outside of tests — use `anyhow::Context` with `?`
- No `clone()` unless the borrow checker demands it — try references first
- No unnecessary `pub` on struct fields
- All build/runtime code stays in Rust — no polyglot tooling in the build (no Bun, Node, Python scripts). Knowledgebase helper scripts under `hoppy-knowledgebase/` are exempt: they're auditable shell snippets, not part of the build.
- New API service coverage goes into `crates/bunny-api/src/<service>/` as a feature-gated module (declare the feature in `crates/bunny-api/Cargo.toml` and add `pub mod <service>;` to `crates/bunny-api/src/lib.rs`). The `bunny-api-<domain>` per-crate pattern was retired in iter-32 — see `decision-log.md`.
- The CLI binary lives in `crates/hoppy-cli/` (package name `hoppy-cli`, binary name `hoppy`).

## PR Discipline
- One iteration = one branch = one PR
- Branch naming: `iter-N/short-description`
- Self-review the diff before requesting review — catch fmt, clippy, dead code yourself

## Integration Tests
Integration tests live in `tests/e2e/` per crate, declared via `[[test]] name = "e2e" path = "tests/e2e/mod.rs"` in `Cargo.toml`. Add new test files as `mod` declarations in `tests/e2e/mod.rs`, not as new top-level files under `tests/` — top-level files become separate binaries and each one re-links the crate from scratch.

## Dogfooding
After (or before) an iteration, build hoppy with `cargo build --release` and use `target/release/hoppy` against a real bunny.net account to dogfood your changes. File friction points as backlog items in `hoppy-knowledgebase/backlog/`. Follow [[dogfooding/dogfooding-playbook]] — never run destructive commands without the safe-prefix + cleanup script. The `live-api` Cargo feature gates the real-API E2E tests (`cargo test --workspace --features live-api --quiet`).
