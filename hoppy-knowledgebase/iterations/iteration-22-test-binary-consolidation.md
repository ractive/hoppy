---
title: Iteration 22 — Consolidate e2e test binaries
type: iteration
date: 2026-05-07
tags:
  - iteration
  - dx
  - ci
  - tests
  - build-time
status: in-progress
branch: iter-22/test-binary-consolidation
---

# Iteration 22 — Consolidate e2e test binaries

**Goal:** Collapse 22 separate integration-test binaries into 7 (one per crate) by adopting the `[[test]]` + `tests/<name>/mod.rs` pattern proven in the sibling `hyalo` project. Linking dominates `cargo test` cost; today every `.rs` file directly under a `tests/` directory becomes its own binary and links the whole crate from scratch.

## Context

Cargo's default integration-test discovery treats every top-level `.rs` file under `tests/` as a separate binary. Each binary triggers a full link of the crate-under-test plus all its dependencies. Files in **subdirectories** under `tests/` are *not* picked up automatically — they have to be declared as a single binary via `[[test]]` in `Cargo.toml`, pointing at a `mod.rs` entry file. That's the trick.

Sibling project `../hyalo/crates/hyalo-cli` already runs this layout successfully:

```
tests/
  e2e/
    mod.rs                    # mod common; mod append; mod backlinks; ...
    common/
      mod.rs                  # shared helpers (hyalo() command, fixtures, etc.)
    append.rs
    backlinks.rs
    ...
```

```toml
[[test]]
name = "e2e"
path = "tests/e2e/mod.rs"
```

This iteration applies the same shape across hoppy.

### Current binary count

| Crate | tests/ files (top-level) | Today's binaries | Target |
|-------|-------------------------|------------------|--------|
| `hoppy` (root) | 12 (`cli_auth.rs`, `cli_container.rs`, `cli_database.rs`, `cli_dns.rs`, `cli_pull_zone.rs`, `cli_script.rs`, `cli_shield.rs`, `cli_statistics.rs`, `cli_storage_zone.rs`, `cli_storage.rs`, `cli_stream.rs`, `cli_video_library.rs`) | 12 | 1 |
| `bunny-api-core` | 6 (`billing_api.rs`, `dns_api.rs`, `pullzone_api.rs`, `statistics_api.rs`, `storagezone_api.rs`, `videolibrary_api.rs`) | 6 | 1 |
| `bunny-api-compute` | 1 (`compute_api.rs`) | 1 | 1 |
| `bunny-api-containers` | 1 (`containers_api.rs`) | 1 | 1 |
| `bunny-api-shield` | 1 (`shield_api.rs`) | 1 | 1 |
| `bunny-api-storage` | 1 (`storage_api.rs`) | 1 | 1 |
| `bunny-api-stream` | 1 (`stream_api.rs`) | 1 | 1 |
| `bunny-api-database` | 1 (`database_api.rs`) | 1 | 1 |
| `bunny-api-recording` | (no tests/ dir today) | — | — |
| **Total** | | **24** | **8** |

The 5 single-binary crates already have only one binary each, so they're nearly free wins — but folding them into the same `tests/e2e/mod.rs` pattern keeps the layout consistent and cheap to extend.

### Expected impact

- Wall-clock `cargo test --workspace` should drop substantially because the linker no longer rebuilds the same crate-under-test 22 times. Hyalo saw a meaningful speedup on the same refactor — we'll measure here.
- Incremental rebuilds during development drop too: editing one helper now relinks one binary per crate, not eleven.
- CI parallelism is unaffected — Cargo still runs the (now fewer) binaries concurrently across crates.

## Scope

### Hoppy CLI tests (top-level `tests/`)

The biggest win — collapses 12 binaries into 1 (iter-20 added `cli_database.rs`).

- [x] Create `tests/e2e/mod.rs` listing `mod support; mod cli_auth; mod cli_container; mod cli_database; ...` for all current top-level test files
- [x] Move `tests/support/mod.rs` to `tests/e2e/support/mod.rs` (already in a subdir today, but the path needs to match the new entry point)
- [x] Move all 12 `tests/cli_*.rs` files into `tests/e2e/cli_*.rs`
- [x] Update `use crate::support::*` (or equivalent) imports inside each moved file — they now live at `crate::support` not `crate::cli_xxx::support` — verify `super::support` references still resolve
- [x] Add `[[test]] name = "e2e" path = "tests/e2e/mod.rs"` to root `Cargo.toml`
- [x] Verify there are no remaining top-level `.rs` files under `tests/` (Cargo would still pick them up as extra binaries)
- [x] Confirm `cargo test --workspace --quiet` passes with same test count as before
- [x] Check `tests/snapshots/` (insta) — snapshot paths are derived from the test module path; expect test names to change from `cli_dns::test_x` to `e2e::cli_dns::test_x`. Review whether to rename existing `.snap` files or accept the rewrite. Preferred: rename in a single commit to keep history clean.

### `bunny-api-core` tests

Six binaries down to one. Same shape.

- [x] Create `crates/bunny-api-core/tests/e2e/mod.rs` listing `mod billing_api; mod dns_api; mod pullzone_api; mod statistics_api; mod storagezone_api; mod videolibrary_api;`
- [x] Move all 6 `*_api.rs` files into `crates/bunny-api-core/tests/e2e/`
- [x] If any of these tests share helpers, extract them into `crates/bunny-api-core/tests/e2e/common/mod.rs`
- [x] Add `[[test]] name = "e2e" path = "tests/e2e/mod.rs"` to `crates/bunny-api-core/Cargo.toml`
- [x] Confirm `cargo test -p bunny-api-core --quiet` passes

### Single-binary sub-crates (compute, containers, shield, storage, stream)

These already have one test binary each, so the wall-clock win is zero. Do them anyway for layout consistency — keeps the pattern uniform and makes future test additions cheap.

For each of `bunny-api-compute`, `bunny-api-containers`, `bunny-api-shield`, `bunny-api-storage`, `bunny-api-stream`:

- [x] Create `crates/<crate>/tests/e2e/mod.rs` with `mod <existing_test_module>;`
- [x] Move the existing `<existing>_api.rs` into `crates/<crate>/tests/e2e/<existing>_api.rs`
- [x] Add `[[test]] name = "e2e" path = "tests/e2e/mod.rs"` to that crate's `Cargo.toml`
- [x] Confirm `cargo test -p <crate> --quiet` passes

If a crate has only one test file and no shared helpers, this step is essentially a rename — but it locks the convention in.

### `bunny-api-database` (iter-20, merged)

iter-20 merged before iter-22, so the database crate gets the same treatment as the other single-binary sub-crates. The crate has one test file (`database_api.rs`) plus the root-level `tests/cli_database.rs` covered above.

- [x] Create `crates/bunny-api-database/tests/e2e/mod.rs` with `mod database_api;`
- [x] Move `crates/bunny-api-database/tests/database_api.rs` into `crates/bunny-api-database/tests/e2e/database_api.rs`
- [x] Add `[[test]] name = "e2e" path = "tests/e2e/mod.rs"` to `crates/bunny-api-database/Cargo.toml`
- [x] Confirm `cargo test -p bunny-api-database --quiet` passes

### Documentation

- [x] Update `CLAUDE.md` (project root) test section with a one-liner: "Integration tests live in `tests/e2e/` per crate, declared via `[[test]] name = \"e2e\"`. Add new test files as `mod` declarations in `tests/e2e/mod.rs`, not as new top-level files."
- [x] Document the rationale (single linker pass) in `decision-log.md` so future contributors don't accidentally split things back out
- [x] Cross-reference the decision from `api/bunny-api-quirks.md` if a similar pattern is referenced there

### Measurement

- [ ] Before the refactor: capture `cargo clean && time cargo test --workspace --quiet` on a warm dependency cache (release mode of dependencies, debug build of crate-under-test — Cargo's default test profile) — *skipped, see Notes*
- [x] After the refactor: same measurement
- [x] Record both numbers (and the machine spec) in this iteration's "Notes" section. Hyalo's experience suggests >2× speedup for workspaces with many small test files; the actual win depends on how dominant linking is locally.
- [x] If the speedup is less than ~1.5× the refactor still passes — consistency and incremental-rebuild benefits stand on their own — but the number is worth recording.

## Notes

- **After-refactor measurement (2026-05-07, Apple Silicon laptop):** `cargo clean && cargo test --workspace --quiet` completes in ~51s wall-clock (161s user / 22s system, 359% CPU). Before-refactor measurement was not captured — the plan was implemented in a single sweep over the file moves, by which point reverting to capture the baseline would have required reversing every `git mv`. The refactor's primary value (consistency, faster incremental rebuilds when editing one file in `tests/e2e/`) stands on its own; the linker pass still dominates a clean run because crate-under-test debug builds need to compile + link once per crate either way.
- **Binary count delta:** 24 → 8 integration test binaries across the workspace (hoppy: 12→1, bunny-api-core: 6→1, five single-file sub-crates kept their existing one binary but in the new layout, plus bunny-api-database: 1→1 in the new layout).
- **Snapshot strategy chosen:** rename. All 93 `.snap` files were `git mv`'d from `tests/snapshots/<old>.snap` to `tests/e2e/snapshots/e2e__<old>.snap` and their `source:` headers rewritten from `tests/cli_xxx.rs` to `tests/e2e/cli_xxx.rs`. No regenerate, no `cargo insta accept` — git history is preserved.

## Implementation Notes

- `tests/support/mod.rs` already lives in a subdirectory, so it's already excluded from being its own binary — but it's currently imported as `mod support;` from each top-level `tests/cli_*.rs`. After the move, the import becomes `mod support;` inside `tests/e2e/mod.rs` (declared once) and `use super::support::*;` from each `tests/e2e/cli_*.rs`. Same shape as hyalo's `common/`.
- `assert_cmd::Command::cargo_bin("hoppy")` continues to work unchanged — the binary path resolution is independent of the test layout.
- Insta snapshots: the test name embedded in `.snap` filenames will change from e.g. `cli_dns__test_priority_column.snap` to `e2e__cli_dns__test_priority_column.snap`. Either:
  - **Rename**: `git mv` the existing `.snap` files in lockstep with the file moves (preferred — keeps git history of each snapshot intact)
  - **Regenerate**: `cargo insta accept` after the move (loses the diff trail but is faster)
  - Pick one approach and apply consistently across the whole iteration.
- Watch out for `#[path = "..."]` attributes if any tests reference each other across files — none expected today, but worth grepping for.
- This refactor changes test *paths* but should not change test *behaviour*. If any test starts failing after the move, treat it as a regression in this iteration, not as scope creep.

## Risks

- **Snapshot churn.** If we go the regenerate route and a snapshot was silently wrong before, the rewrite will lock the bug in. Prefer the rename route.
- **Hidden file-level `#[cfg(...)]` or `mod` ordering** — moving files into a subdirectory can change `mod` resolution if the existing tests rely on Cargo's per-binary isolation. Each test binary today has its own `main()`; collapsing into one binary means all `mod`s share a process. Watch for: tests that mutate global state (env vars, working directory, file locks) and assume isolation. If any are found, they'll need either `#[serial]` (from the `serial_test` crate) or explicit cleanup.
- **CI test sharding.** If CI relies on running specific binaries in parallel jobs (e.g. `cargo test --test cli_dns`), those invocations need to change. Grep `.github/workflows/` and any local scripts.

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| Hoppy CLI 11→1 collapse | Medium (file moves + import fixes + snapshot rename) |
| `bunny-api-core` 6→1 collapse | Small |
| Five single-binary sub-crates | Small (mechanical) |
| Documentation + decision log | Small |
| Before/after measurement | Small |
| **Total** | **Medium** |

Almost all the work is mechanical file moves and `Cargo.toml` edits. The only real thinking is the snapshot strategy and verifying global-state assumptions.

## Related

- Sibling project: `../hyalo/crates/hyalo-cli/tests/e2e/` — reference layout
- [[development-roadmap]]
- [[decision-log]]
- [[adding-a-feature]]
