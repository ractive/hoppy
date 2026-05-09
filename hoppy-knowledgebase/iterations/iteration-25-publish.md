---
title: Iteration 25 — Publish hoppy
type: iteration
date: 2026-05-07
tags:
  - iteration
  - release
  - ci-cd
  - packaging
  - crates-io
  - homebrew
status: planned
branch: iter-25/publish
---

# Iteration 25 — Publish hoppy

**Goal:** Bring hoppy's release pipeline up to the bar set by `../hyalo` and `../ff-rdp` so it can be published cleanly to crates.io, Homebrew, Scoop, and (optionally) winget. Hoppy already builds release artifacts for all platforms — what's missing is the surrounding hygiene (version-check, security audit, SHA-pinned actions, package-manager publishing, Homebrew tap maintenance).

This iteration **assumes [[iterations/iteration-23-hyalo-best-practices]] has merged** because section 9 of iter-23 (move CLI binary to `crates/hoppy-cli/`) is a hard prerequisite for clean crates.io publishing. If that section was deferred from iter-23, do it as part of this iteration before anything else.

## Context

Hoppy's `release.yml` today (307 lines) covers the build matrix, shell completions, man pages, deb/rpm packaging, and uploads artifacts to a GitHub release. Hyalo's `release.yml` (434 lines) and ff-rdp's (439 lines) add: pre-build version verification, pre-build security audit (`cargo audit` + `cargo deny`), SHA-pinned third-party actions, crates.io publishing of both the API crates and the CLI crate, Homebrew formula updating in a tap repo, Scoop manifest, and winget submission (ff-rdp only).

Reference workflows:
- `../hyalo/.github/workflows/release.yml` — full reference for crates.io + Homebrew
- `../ff-rdp/.github/workflows/release.yml` — adds Scoop + winget on top

Today's hoppy release matrix already builds: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` (cross), `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`. Keep this set.

## Scope

### 1. Pre-publish hygiene (hard pre-requisites)

Several things must be true before the project can be published. Audit each:

- [ ] **Crate names are unique on crates.io.** Check `cargo search hoppy bunny-api-core bunny-api-compute bunny-api-containers bunny-api-database bunny-api-recording bunny-api-shield bunny-api-storage bunny-api-stream` — confirm none are taken by an unrelated package. If `hoppy` is taken, pick a fallback (`bunny-hoppy`?) and document the choice. The `bunny-api-*` names are likely free but verify each.
- [ ] **Version bump from 0.1.0 to a release version.** Decide: stay at `0.1.0` for the first publish (signals early-but-stable), or jump to `0.1.0` minor cycle? Hyalo is at `0.14.0`; ff-rdp's version is whatever is current. Recommendation: ship as `0.1.0` (first public) and bump from there. Document the version-bump rule (semver) in `decision-log.md`.
- [ ] **Every `Cargo.toml` has a `description`, `license`, `repository`, `homepage`, `readme`, `keywords`, `categories`.** The root has these; sub-crates may not. Audit and fix. crates.io rejects publishes missing required fields.
- [ ] **README polish.** Hoppy's `README.md` exists; before publishing, audit it as the package landing page on crates.io and the Homebrew formula description. Sections needed: install (Homebrew + cargo + binary download), quickstart, command map, configuration (BUNNY_API_KEY env var), troubleshooting, link to docs.
- [ ] **CHANGELOG.md** (set up in iter-23) gets a `## [0.1.0] - YYYY-MM-DD` entry summarising every iteration that shipped.
- [ ] **LICENSE present and matches Cargo.toml `license = "MIT"`** — already true; verify.
- [ ] **Crates publish in dependency order**: `bunny-api-core` first (no internal deps), then `bunny-api-{compute,containers,database,recording,shield,storage,stream}` (each depends on `bunny-api-core` only? verify), then `hoppy` (or `hoppy-cli` after iter-23 §9 lands). The release workflow must publish in this order or `cargo publish` fails.
- [ ] **`[workspace.package]` hoist** (from iter-23 §3) makes version-bumping a single edit. Confirm it's in place.

### 2. Release workflow upgrades — version + security gates

Port the pre-build jobs from hyalo's release.yml.

- [ ] **Add `version-check` job**: parse `${{ github.event.release.tag_name }}`, strip leading `v`, compare to `cargo metadata --no-deps | jq` extraction of the `hoppy` (or `hoppy-cli`) crate version. Fail the build if they differ. Lift hyalo's job verbatim, change the package name.
- [ ] **Add `security` job**: install `cargo-audit` and `cargo-deny`, run `cargo audit` (vulnerabilities) and `cargo deny check` (licences/advisories/bans/sources). Lift hyalo's job verbatim.
- [ ] **Add a `deny.toml`**: copy `../hyalo/deny.toml` as a starting template, customise the licence allow-list and advisory ignore-list to match hoppy's actual deps. Run `cargo deny check` locally first to surface needed exemptions.
- [ ] **Make `build` jobs depend on both** via `needs: [version-check, security]`. The whole pipeline aborts if either fails.

### 3. Release workflow upgrades — action SHA pinning

Today hoppy uses `actions/checkout@v4`, `Swatinem/rust-cache@v2`, etc. Hyalo and ff-rdp pin to commit SHAs (`actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5 # v4.3.1`). This is GitHub's documented supply-chain recommendation.

- [ ] **Pin every third-party action**. Use the SHAs already chosen by hyalo as a starting point (they've been vetted and used in production). For new actions, look up the latest SHA via `gh api repos/<owner>/<repo>/git/ref/tags/<version>` and add a `# vX.Y.Z` comment after the SHA.
- [ ] Apply the same pinning to `ci.yml` (not just `release.yml`).

### 4. Crates.io publishing

- [ ] **Add `CARGO_TOKEN` repository secret** via `gh secret set CARGO_TOKEN` (the user does this once with their crates.io API token).
- [ ] **Add a `verify-cargo-token` step** before publishing that fails fast with a clear message if the secret is missing. Lift from hyalo.
- [ ] **Add a `publish-crates` job** (post-build, post-upload). Steps:
  - Publish each `bunny-api-*` crate in dependency order — use `cargo publish -p <crate> --token "${{ secrets.CARGO_TOKEN }}"`.
  - Then publish `hoppy` (or `hoppy-cli`).
  - Between each publish, sleep 30s to let crates.io's index propagate so the next dependent crate can resolve.
- [ ] **Test the publish path with `--dry-run` first**: in a separate workflow run (not on tag), do `cargo publish -p <each crate> --dry-run --token "${{ secrets.CARGO_TOKEN }}"`. Confirms each crate has all required metadata.
- [ ] **Decide what NOT to publish**: `xtask` (workspace tooling, not a public crate) gets `publish = false` in its `Cargo.toml`.

### 5. Homebrew formula + tap

Hyalo and ff-rdp both maintain a `homebrew-tap` repo (e.g. `ractive/homebrew-tap`) with a Formula file that the release workflow updates on each release.

- [ ] **Create `ractive/homebrew-tap` repo** if it doesn't exist (the user owns ractive). Copy `../hyalo/...` formula structure as a starting template.
- [ ] **Add `HOMEBREW_TAP_TOKEN` secret**: a fine-grained PAT with `contents: write` on the tap repo.
- [ ] **Add the "Update Homebrew formula" job** to release.yml (lift from hyalo). Steps:
  - Download `SHA256SUMS` from the GitHub release
  - Update the formula file (`Formula/hoppy.rb`) with new version + per-target SHA256s + tarball URLs
  - `git push` to the tap repo
- [ ] **Test the formula locally before relying on it in CI**: `brew install --build-from-source ./Formula/hoppy.rb` after a manual edit to verify the formula's structure works for hoppy.
- [ ] **Document the install path** in README: `brew tap ractive/tap && brew install hoppy`.

### 6. Scoop manifest (Windows)

ff-rdp has Scoop publishing; hyalo does too. Same pattern.

- [ ] **Create `ractive/scoop-bucket` repo** if needed (or reuse an existing one).
- [ ] **Add `SCOOP_BUCKET_TOKEN` secret**.
- [ ] **Lift the "Update Scoop manifest" job** from `../ff-rdp/.github/workflows/release.yml`. Updates `bucket/hoppy.json` with new version + SHA256 + zip URL.
- [ ] **Document install path** in README: `scoop bucket add ractive https://github.com/ractive/scoop-bucket && scoop install hoppy`.

### 7. winget submission (optional, later)

ff-rdp includes a winget-pkgs PR step. This is more involved (forks `microsoft/winget-pkgs`, creates a manifest PR per release, requires Microsoft review). Defer unless the user wants Windows users to find hoppy via `winget install`.

- [ ] **Decision point**: include winget in iter-25 or defer to a follow-up? Recommendation: defer. Scoop covers most Windows power-users; winget submission has Microsoft review delays and rejection paths. Land iter-25 without it, add as a later iteration.
- [ ] If included: lift the winget job from ff-rdp verbatim, change package name + identifier.

### 8. SHA256SUMS aggregation

- [ ] **Add the "Generate SHA256SUMS" step** post-build, pre-upload. Lift from hyalo. Concatenates per-target sha256 into a single `SHA256SUMS` file uploaded as a release asset. Homebrew/Scoop jobs depend on this file.

### 9. CLI smoke test in the build matrix

Hyalo runs `cargo run --release --target ${{ matrix.target }} -p hyalo-cli -- --help` after the build to confirm the binary actually executes (catches dynamic-link issues, missing runtime deps).

- [ ] **Add the smoke test** for native targets (skip for cross-compiled aarch64-linux which can't run on the runner).
- [ ] **Pipe the help output** to a check that grep-matches expected subcommands — confirms the CLI didn't accidentally lose a major command.

### 10. Release announcement + rollout plan

- [ ] **Document the publish sequence** in `hoppy-knowledgebase/release/release-checklist.md`:
  - Pre-flight: every section above ticked, CHANGELOG up-to-date, version bumped
  - Cut: tag `v0.1.0`, push, create GitHub release, paste CHANGELOG into release notes
  - Watch: release.yml succeeds (version-check + security + build matrix + publish-crates + Homebrew + Scoop)
  - Verify: `cargo install hoppy` works, `brew install ractive/tap/hoppy` works, `scoop install hoppy` works (on a Windows machine)
  - Announce: README badge update, README install section update, social/blog post if desired
- [ ] **Plan the post-release dogfood**: install via Homebrew on a clean Mac, run the section-5 dogfooding playbook from iter-23, file every friction point as a backlog item.

### 11. Branding consistency

- [ ] **Confirm the project name everywhere**: `hoppy` (lowercase) in Cargo.toml, README, formula, manifest. The `bunny-api-*` crates are subpackages. Document both names in README.
- [ ] **Confirm GitHub repo URL**: `github.com/ractive/hoppy`. README install instructions, Cargo.toml `repository`, formula + manifest URLs all match.

## Implementation Notes

- **Big-bang vs incremental**: option A — land everything in one PR, then cut `v0.1.0` immediately. Option B — land sections 1–4 (hygiene + crates.io publish path), do a `--dry-run` rehearsal, then add 5–9 in a follow-up iteration. Recommendation: option B. The `--dry-run` rehearsal exposes metadata gaps that you'd rather find before tagging.
- **Tag format**: `v0.1.0` (lowercase v + semver). Hyalo uses this; the release workflow's version-check regex strips the leading `v`.
- **Don't publish until iter-19 PR feedback is fully addressed**. iter-19 introduced enum forward-compatibility for unknown bunny.net variants — that's a behaviour-shaping change worth one more PR-review cycle before locking it into `v0.1.0`.
- **Homebrew tap repo can be created reactively**: the workflow can `gh repo create` it on first run. But the secrets (token) need to be set up in advance.
- **Document credential ownership**: who owns the `CARGO_TOKEN` (the project owner's crates.io account) and the tap-token. Note in `release-checklist.md` so a future contributor knows where the secrets live.

## Suggested test cases

1. **Tag mismatch**: push tag `v0.99.99` while Cargo.toml is `0.1.0`. The `version-check` job fails before any build runs.
2. **Vulnerable dep**: pin a known-bad version of a dep transiently. The `security` job (`cargo audit`) fails.
3. **Dry-run publish**: `cargo publish --dry-run -p bunny-api-core` succeeds with all metadata present.
4. **Homebrew formula**: `brew install --build-from-source ./Formula/hoppy.rb` (after a manual SHA update) installs hoppy and `which hoppy` returns a sane path.
5. **End-to-end on a clean tag**: tag `v0.1.0`, the workflow runs to completion, all artifacts and packages exist, `cargo install hoppy` works on a machine that's never had hoppy before.

## Risks

- **Crate name collision on crates.io**: `hoppy` is a short name; might already exist. Have a fallback ready (`bunny-hoppy`?). Same risk for `bunny-api-*`.
- **crates.io publish ordering**: each dependent crate must wait for the index to propagate. The 30s sleep usually works but sometimes needs longer.
- **Homebrew formula bitrot**: as bunny.net's API changes, hoppy's behaviour changes; users on Homebrew may lag. Document the upgrade path (`brew upgrade hoppy`).
- **Cross-platform packaging surprises**: aarch64-linux uses `cross` (Docker); Windows uses MSVC; macOS has the universal-vs-per-arch question. Today hoppy ships per-arch; document this is intentional (no fat binaries).
- **Secret leakage**: `CARGO_TOKEN` in CI must be scoped to publishing the specific crates only — don't reuse a token with broad permissions.

## Estimated Complexity

| Topic | Complexity |
|-------|------------|
| Pre-publish hygiene (1) | Medium |
| Version + security gates (2) | Small (lift verbatim) |
| Action SHA pinning (3) | Small (mechanical) |
| Crates.io publishing (4) | Medium |
| Homebrew formula + tap (5) | Medium |
| Scoop manifest (6) | Small (lift from ff-rdp) |
| winget (7) | Deferred / Large if attempted |
| SHA256SUMS aggregation (8) | Small |
| CLI smoke test (9) | Small |
| Release checklist + rollout (10) | Small |
| Branding consistency (11) | Small |
| **Total** | **Medium–Large** |

The bulk of the work is mechanical lifts from hyalo + ff-rdp. The actual decisions are: which version to publish as, which name to claim on crates.io, and whether to defer winget.

## Related

- Reference workflows: `../hyalo/.github/workflows/release.yml`, `../ff-rdp/.github/workflows/release.yml`
- Reference deny config: `../hyalo/deny.toml`
- Pairs with: [[iterations/iteration-23-hyalo-best-practices]] (its §9 may be a hard pre-req)
- [[development-roadmap]]
- [[decision-log]]
- existing: `[[release/release-setup-checklist]]` if present (iter-8 covered earlier release readiness)
