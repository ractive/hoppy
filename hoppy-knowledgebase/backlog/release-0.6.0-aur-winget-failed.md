---
title: v0.6.0 release — AUR and winget publications failed (retry needed)
type: backlog
date: 2026-08-10
origin: release-v0.6.0
priority: high
status: open
tags:
  - backlog
  - release
  - packaging
  - aur
  - winget
---

# v0.6.0: AUR + winget publications failed

Release run 31338041159 (v0.6.0, 2026-08-09) went green overall, but both
new publication channels failed. Because `aur` and `winget` are
`continue-on-error`, the run's green checkmark hid both failures.

## winget — blocked on token scope (action: james)

`gh repo sync ractive/winget-pkgs` fails before the version PR is even
prepared:

> Upstream commits contain workflow changes, which require the `workflow`
> scope or permission to merge.

**Fix:** regenerate `WINGET_TOKEN` with the `workflow` scope (classic PAT)
or "Workflows: read/write" (fine-grained), update the repo secret, then
rerun the job. Every future release hits this until the token is fixed.

## AUR — pipeline works, AUR git service was in maintenance

The job generates and commits the `hoppy-bin` 0.6.0 PKGBUILD + .SRCINFO
correctly and the SSH key authenticates, but the push gets:

> The AUR is down due to maintenance. We will be back soon.

Confirmed independently: `ssh aur@aur.archlinux.org` logs in fine
("Welcome to AUR, ractive!") while `git ls-remote
ssh://aur@aur.archlinux.org/hoppy-bin.git` still returns the maintenance
banner — their git service lagged the SSH frontend. Watched ~20 min
(3 rerun attempts, 22:14–22:32 UTC); still down.

**Fix:** once AUR git is back (`git ls-remote
ssh://aur@aur.archlinux.org/hoppy-bin.git` lists refs), rerun:

```bash
gh run rerun --job "$(gh run view 31338041159 --json jobs \
  -q '.jobs[] | select(.name | test("aur")) | .databaseId')"
```

Then verify <https://aur.archlinux.org/packages/hoppy-bin> exists (first
publish creates the package).

## Follow-up worth considering

`continue-on-error` on aur/winget means a green release run does not mean
"published everywhere". Candidate improvement in ractive/release-workflows:
a trailing job that lists any failed publication jobs in the run summary
(still non-blocking, but visible).

## Related

- [[release/release-checklist]]
