---
title: git history contains dead DeploymentKeys in compute fixtures
type: backlog
date: 2026-07-10
origin: fixture-refresh sweep 2026-07-10
priority: low
status: resolved
tags:
  - security
  - fixtures
  - git-history
---

# git history contains dead DeploymentKeys in compute fixtures

## What

`fixtures/compute/script_get.json` has carried a **real** `DeploymentKey`
value since the 2026-05-14 fixture refresh (`f122e10e-…`), because the
record-mode redaction rules matched `apikey`/`accesskey`/`signingkey` but
not `deploymentkey`. The 2026-07-10 sweep caught this and fixed the rule
(plus bare `Key` and the 72-char double-UUID account-key value shape), so
fixtures going forward hold `"<redacted>"`.

## Risk

Low. The keys belong to `hpscv-*` edge scripts created and deleted inside
the live-test lifecycle — the resources no longer exist, so the keys are
dead. But the values remain in git history on the public repo.

## Options

- Accept (keys are dead, test account only) — likely fine.
- Rewrite history (`git filter-repo`) — disruptive, probably not worth it.
- Rotate/delete anything on the test account that could still honour an
  old deployment key (already implied by resource deletion).

## Related

- [[dogfooding/dogfooding-playbook]] — redaction rules
- [[iterations/iteration-48-record-pii-redaction]] — original redaction layer

## Resolution (2026-08-09)

**Accepted.** The leaked DeploymentKeys belong to `hpscv-*` edge scripts
created and deleted inside the live-test lifecycle — the resources no
longer exist, so the keys authenticate nothing. The redaction rule gap
that caused the leak was fixed in the 2026-07-10 sweep (deploymentkey,
bare Key, and double-UUID shapes now redact). A `git filter-repo` history
rewrite would disrupt every clone/fork for no security gain. No further
action.
