---
title: Release Setup Checklist
date: 2026-03-18
tags:
  - release
  - setup
  - one-time
status: completed
type: checklist
---

# Release Setup Checklist

One-time setup steps before the first release.

## GitHub Repository

- [x] Ensure `ractive/hoppy` repo is public (or at least has releases enabled)
- [x] Add LICENSE file to the repo root

## Homebrew Tap

- [x] Create public repo `ractive/homebrew-tap` on GitHub
- [x] Create `Formula/` directory in the tap repo
- [x] Create a **fine-grained** GitHub Personal Access Token:
  1. Go to <https://github.com/settings/personal-access-tokens/new>
  2. Token name: `hoppy-homebrew-tap`
  3. Expiration: 1 year (or custom — you'll need to rotate it)
  4. Resource owner: `ractive`
  5. Repository access: **Only select repositories** → select `ractive/homebrew-tap`
  6. Permissions:
     - **Contents**: Read and write (needed to push the updated formula)
     - Everything else: No access
  7. Click **Generate token** and copy it
- [x] Add the token as a repository secret in `ractive/hoppy`:
  1. Go to <https://github.com/ractive/hoppy/settings/secrets/actions>
  2. Click **New repository secret**
  3. Name: `HOMEBREW_TAP_TOKEN`
  4. Value: paste the token
  5. Click **Add secret**

## winget (post-release)

- [x] Bootstrap PR under review at `microsoft/winget-pkgs#400670` — winget is
  **not** live yet, so don't advertise a winget install until it merges — merged 2026-07-24; winget is live
- [x] Manifest should use `InstallerType: zip`, `NestedInstallerType: portable`
- [x] Package identifier: `ractive.hoppy`

## Releasing a new version

The per-release steps now live in [[release-checklist]] — follow that. In
short: bump the workspace version, add the `CHANGELOG.md` entry, then
`gh release create vX.Y.Z --generate-notes`. The shared reusable workflow in
[ractive/release-workflows](https://github.com/ractive/release-workflows)
(a thin caller in `.github/workflows/release.yml`) builds the full 7-target
matrix — including static musl Linux binaries — attaches archives, `.deb`/`.rpm`,
SBOMs, and build-provenance attestations, publishes the crates, and updates the
Homebrew tap, Scoop bucket, and the hosted apt/yum repos at Cloudsmith.

## Related

- [[research/release-engineering-research]] — research behind release decisions
- [[development-roadmap]] — iteration 8 (release workflow)
- [[decision-log]] — release & packaging decisions
