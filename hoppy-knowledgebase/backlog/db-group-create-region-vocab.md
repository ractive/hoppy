---
title: db group create region flags use two vocabularies with no local validation
type: backlog
date: 2026-08-09
tags: [backlog, database, dx, validation]
status: open
priority: low
origin: dogfooding-2026-08-09
---

# `db group create` region flags mix two vocabularies, no local validation

`--primary-region` takes short uppercase codes (`DE`, `AMS`, ...);
`--storage-region` takes AWS-style ids (`eu-west-1`, `us-east-1`). Help
text documents neither vocabulary, values are case-sensitive, and errors
come raw from the API — lowercase `de` for primary returns a JSON-schema
dump; `DE` for storage returns a 500 "This storage region DE is not
supported." Both vocabularies are available locally via
`db config show`, so the CLI could validate before the call (mirroring
what `db create` does for slugs) or at least name the valid values in
help text/possible-values.
