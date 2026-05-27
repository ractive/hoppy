---
title: >-
  storage upload/rm display path with `zone//file` when --remote-path has a
  leading slash
type: backlog
date: 2026-05-27
status: resolved
priority: low
origin: dogfooding-2026-05-27 (post-iter-39)
resolved-in: iter-40
---

# Double-slash in storage display paths

`hoppy storage upload --zone <zone> --file <local> --remote-path /foo.txt`
prints:

```
Uploaded /tmp/dogfood.txt → hpst-1778785767589-1//dogfood-2026-05-27.txt
                                                ^^
```

Same for `storage rm`:

```
Deleted hpst-1778785767589-1//dogfood-2026-05-27.txt
```

The API resolves `zone//path` the same as `zone/path`, so the operation
succeeds. The display string is just unjoined: `<zone> + "/" + <remote>`
where `<remote>` already starts with `/`.

## Fix

In `crates/hoppy-cli/src/commands/storage.rs` (the upload/rm/download
success messages), trim leading `/` from `remote_path` before joining:

```rust
let display = format!("{zone}/{}", remote_path.trim_start_matches('/'));
```

Or, alternatively, normalise the user input: strip the leading slash on
clap parse (`value_parser`) so the internal representation is always
slash-less.

## Out of scope

The underlying HTTP request is fine — this is a display-only fix.
