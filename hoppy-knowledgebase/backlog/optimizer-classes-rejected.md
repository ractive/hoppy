---
title: pull-zone update --optimizer-classes always rejected as model.invalid
type: backlog
date: 2026-05-10
status: completed
priority: medium
origin: dogfooding-2026-05-10
---

# Optimizer classes wire format

Iter-26 added `--optimizer-classes <JSON>` on `pull-zone update`. Every JSON
value tested gets a 400:

```sh
hoppy pull-zone update --id <id> --optimizer-classes '{}'
hoppy pull-zone update --id <id> --optimizer-classes '{"thumb":"width=200,quality=80"}'
# both → bunny.net API error 400 (model.invalid): Model validation failed
```

`--debug` shows the request goes to `POST /pullzone/<id>` and the body is
rejected without a useful field name (`Field: id`).

The serde helper `deserialize_string_lossy_option` documents that the API
returns `OptimizerClasses` as a *JSON-encoded string of a map* on the
response — likely the request side expects the same: a string-quoted JSON
blob, not a raw object. So the CLI's `optimizer_classes(cls.as_str())` call
in `src/commands/pull_zone.rs:232` may be sending a literal `{...}` where
the API wants `"{...}"` (string-escaped).

## Action items

- Confirm wire format with bunny docs (https://docs.bunny.net/docs/optimizer-classes
  per the help text).
- If the API really does want the doubly-encoded form, do the encoding
  client-side so the user passes a normal JSON map.
- Add a live-api e2e test that round-trips one class definition.
- Improve the error: when the API replies with `Field: id` for an unknown
  pullzone field, hoppy should surface the offending field if possible.
