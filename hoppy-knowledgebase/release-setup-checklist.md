---
title: "Release Setup Checklist"
date: 2026-03-18
tags:
  - release
  - setup
  - one-time
status: active
---

# Release Setup Checklist

One-time setup steps before the first release.

## GitHub Repository

- [x] Ensure `ractive/hoppy` repo is public (or at least has releases enabled)
- [x] Add LICENSE file to the repo root

## Homebrew Tap

- [x] Create public repo `ractive/homebrew-hoppy` on GitHub
- [x] Create `Formula/` directory in the tap repo
- [ ] Create a **fine-grained** GitHub Personal Access Token:
  1. Go to https://github.com/settings/personal-access-tokens/new
  2. Token name: `hoppy-homebrew-tap`
  3. Expiration: 1 year (or custom — you'll need to rotate it)
  4. Resource owner: `ractive`
  5. Repository access: **Only select repositories** → select `ractive/homebrew-hoppy`
  6. Permissions:
     - **Contents**: Read and write (needed to push the updated formula)
     - Everything else: No access
  7. Click **Generate token** and copy it
- [ ] Add the token as a repository secret in `ractive/hoppy`:
  1. Go to https://github.com/ractive/hoppy/settings/secrets/actions
  2. Click **New repository secret**
  3. Name: `HOMEBREW_TAP_TOKEN`
  4. Value: paste the token
  5. Click **Add secret**

## winget (post-release)

- [ ] After v0.1.0 is published, create a winget manifest and submit PR to `microsoft/winget-pkgs`
- [ ] Manifest should use `InstallerType: zip`, `NestedInstallerType: portable`
- [ ] Package identifier: `ractive.hoppy`

## Releasing a new version

1. Update version in `Cargo.toml` (root package)
2. Update `CHANGELOG.md` with new section
3. Commit: `git commit -m "Release vX.Y.Z"`
4. Tag: `git tag vX.Y.Z`
5. Push: `git push && git push --tags`
6. The release workflow handles everything else:
   - Builds 6 target binaries
   - Creates .tar.gz / .zip archives with completions + man pages
   - Builds .deb and .rpm packages
   - Creates GitHub Release with all artifacts + checksums
   - Updates Homebrew tap formula
