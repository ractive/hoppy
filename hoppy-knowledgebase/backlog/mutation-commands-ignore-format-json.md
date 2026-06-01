---
title: Mutation commands ignore `--format json` and always print prose
type: backlog
date: 2026-05-31
status: resolved
priority: medium
origin: dogfooding-2026-05-31
tags:
  - cli
  - json
  - dx
  - scripting
  - consistency
---

# `--format json` is silently ignored on mutation subcommands

Several mutation subcommands print human-readable confirmation lines
regardless of the `--format` value. Scripts that pipe `hoppy … --format
json | jq …` break with `parse error: Invalid numeric literal at line 1`.

## Observed during dogfooding 2026-05-31

```sh
# Returns the prose line, not JSON:
hoppy pull-zone edge-rule add --id 5940331 \
  --description 'dogfood edge rule' \
  --action-type set-response-header \
  --action-param1 X-Dogfood --action-param2 yes \
  --trigger 'url:https://example.com/test/*' \
  --format json
# → "Added edge rule to pull zone 5940331"

hoppy pull-zone edge-rule enable --id 5940331 \
  --rule-id <guid> --enabled false --format json
# → "Disabled edge rule <guid> on pull zone 5940331"

hoppy pull-zone edge-rule delete --id 5940331 \
  --rule-id <guid> --yes --format json
# → "Deleted edge rule <guid> from pull zone 5940331"
```

Same surface, the read-only `pull-zone edge-rule list --format json`
correctly returns a JSON array.

## Why this matters

- `jq` chains break on the first prose line — no useful error message,
  just `parse error`.
- The text confirmation has no machine-readable handle for the rule GUID
  or the action that was applied, so a script that creates a rule and
  then wants to act on it has to re-issue `list` and grep.

## Likely scope

`pull-zone edge-rule {add,update,enable,delete}` are the smoking gun.
The same pattern likely affects other mutation surfaces — worth
checking:

- `pull-zone hostname add/remove/load-free-cert/set-force-ssl`
- `pull-zone referrer add/remove`
- `pull-zone ip add/remove`
- `storage rm`, `storage upload`, `storage download`
- `dns record add/update/delete`
- `shield zone create/update/delete`
- `db create/delete/fork/restore`
- `purge`

## Suggested fix shape

When `--format json` is set, emit a small JSON object summarising the
result, e.g.

```json
{"status":"ok","action":"add","resource":"edge-rule","pullZoneId":5940331,"guid":"4309cc85-…"}
```

For `delete`, include the GUID/id that was deleted. For `add`, include
the new GUID so scripts can chain. For `enable`, echo the new enabled
state. Tabular output stays as-is.

## Related

- [[json-output-casing-inconsistency]]
- [[debug-flag-omits-request-body]]
