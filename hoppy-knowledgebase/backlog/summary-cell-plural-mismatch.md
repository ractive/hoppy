---
title: "`<1 items>` should be `<1 item>` — singular/plural mismatch in get summary cells"
type: backlog
date: 2026-05-28
status: planned
priority: low
origin: dogfooding-2026-05-28 (post-iter-40)
---

# Summary-cell pluralisation

iter-40 introduced summary cells for nested array/object values in
single-resource `get` tables. The array summary doesn't account for the
n=1 case:

```
| Hostnames | <1 items> |
```

Should read `<1 item>`. Two-line fix in the renderer:

```rust
let label = if n == 1 { "item" } else { "items" };
format!("<{n} {label}>")
```

(Or use the same logic for typed labels: `<1 hostname>` / `<3 hostnames>`
if the field name is known.)

## Out of scope

- The object-summary form `<object: N fields>` has the same edge case
  but `fields` reads fine as a label even at n=1, so it's not pressing.
