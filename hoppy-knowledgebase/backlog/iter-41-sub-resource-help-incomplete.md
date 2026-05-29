---
title: "iter-41 #3: sub-resource `--id` help-text audit incomplete (only `edge-rule list` was fixed)"
type: backlog
date: 2026-05-29
status: resolved
priority: medium
origin: dogfooding-2026-05-29 (post-iter-41)
resolved-by: "[[iterations/iteration-42-dogfooding-2026-05-29-fixes]]"
---

# iter-41 #3 missed three of the four examples in its own plan

The iter-41 plan ([[../iterations/iteration-41-dogfooding-2026-05-28-polish]]
§3) explicitly listed four sub-resource commands as targets:

```
- `pull-zone edge-rule list`   → help = "Pull zone ID"
- `pull-zone hostname add`     → help = "Pull zone ID"
- `container endpoint list`    → help = "Container app ID"
- `dns record list`            → help = "DNS zone ID"
```

Only the first one got fixed. The other three still ship undocumented:

```
hoppy pull-zone hostname add --help
      --id <ID>            <-- no help text

hoppy container endpoint list --help
      --app-id <APP_ID>    <-- no help text

hoppy dns record list --help
      --zone-id <ZONE_ID>  <-- no help text
```

Two of these aren't even `--id` — they use `--app-id` and `--zone-id`,
which is a related inconsistency tracked separately (see
[[parent-resource-arg-name-inconsistency]]).

## Suggested fix

Add the `help = "..."` attributes that iter-41 §3 already specified,
plus any other sub-resource args surfaced by the broader audit:

```sh
grep -rn -B1 -A3 '#\[arg' crates/hoppy-cli/src/cli/ | grep -B3 '"id"\|"app_id"\|"zone_id"\|"library_id"' | head -40
```

A quick once-over of the broader CLI is worth doing here because the
original iter-41 audit clearly missed cases.

## Related

- [[../iterations/iteration-41-dogfooding-2026-05-28-polish]] — original
  (incomplete) audit
- [[parent-resource-arg-name-inconsistency]] — the `--id` vs `--app-id`
  vs `--zone-id` divergence is a separate concern
