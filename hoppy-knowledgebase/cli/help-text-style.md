---
title: LLM-friendly help text style
type: docs
date: 2026-05-09
status: active
tags: [cli, help-text, llm, style-guide]
---

# Help text style — humans and agents

Hoppy's primary consumers are humans **and** LLMs. Both pipe `hoppy <noun> --help` and try to understand it. Help text that's good for one is usually good for the other; the rules below codify what "good" means.

## The four-part template

Every command's `clap::Command` should set:

```rust
#[command(
    about = "<one-line summary>",
    long_about = "<multi-line semantic description>",
    after_help = "<at least one example, plus cross-references>",
)]
```

### 1. `about` — one-line summary

- Imperative mood: "Create a pull zone", not "Creates" or "Create..."
- Mention the noun the command operates on
- ≤ 80 chars so it fits in the parent `--help` listing
- For destructive commands, lead with the warning marker: `"Delete a pull zone (DESTRUCTIVE)"`

### 2. `long_about` — semantic description

What `--help` shows when `-h` is replaced by `--help`. Include:

- **What the command does**, in the bunny.net domain language (use the same nouns as dash.bunny.net)
- **Edge cases / surprises** the API has (e.g. "creating a pull zone also implicitly creates a Pull Zone hostname under b-cdn.net")
- **Enum semantics** — every `--type`, `--mode`, `--codec` flag must list possible values *with their meanings*, not just the values
- **ID flags** — every `--*-id` mentions how to discover IDs: `"use \`hoppy <noun> list\` to find IDs"`
- **Destructive prefix** — destructive commands start with `"DESTRUCTIVE: <what gets removed>. Requires --yes or interactive confirm."`
- No decorative ASCII art — LLMs parse plain text reliably; box drawing breaks the parse

### 3. `after_help` — examples + cross-refs

- At least **one realistic example** showing the most common invocation
- A second example for any non-trivial flag combination
- Cross-references in the form `See also: hoppy <related>`. E.g. on `pull-zone create`: `See also: hoppy pull-zone hostname add, hoppy pull-zone edge-rule add`
- For a destructive command, mention the safe inverse: `See also: hoppy <noun> list (find what to delete)`

### 4. Argument descriptions

Every `#[arg]` gets a `help` *and* (for non-trivial flags) a `long_help`:

- `help`: ≤ 60 chars, fits in the column layout
- `long_help`: explains units (bytes? seconds? a region code? a CIDR?), default behavior, and interaction with other flags
- Mark required flags as required in clap so the auto-generated help shows them. Don't rely on description text to convey "required".

## LLM-specific rules

Apart from "no decorative ASCII art":

- **Consistent indentation** — clap's default is fine; don't override with custom formatters that break alignment.
- **Stable structure** — every command's help has the same shape (Usage / Options / Arguments / Examples), so an LLM can extract semantically.
- **Possible values inline** — when clap auto-prints `[possible values: a, b, c]`, also explain each in `long_help`. The list alone tells the LLM nothing about semantics.
- **Crosslinks resolve** — every "See also: hoppy X Y" must be a real, runnable command. An LLM will execute these.
- **Don't dump JSON schemas** in help. Reference the bunny.net API docs URL instead.

## Drill-down hints (deferred)

Hyalo's iter-107 added per-command "drill-down hints": after the human result, the CLI prints `tip: hyalo find --tag X` style next-step suggestions. Hoppy doesn't have this yet — it's a backlog item from iter-23 (see [[../backlog/]]).

## Smell checklist

If a command's `--help` fails any of these, fix it:

- [ ] One-line `about`?
- [ ] `long_about` mentions every flag's semantic role?
- [ ] At least one `after_help` example?
- [ ] Cross-references to related subcommands?
- [ ] Enum flags list value meanings, not just values?
- [ ] ID flags say where to find IDs?
- [ ] Destructive ops prefixed with "DESTRUCTIVE:"?
- [ ] Free of decorative ASCII / box-drawing?

## Related

- [[command-tree]]
- [[../decision-log]]
- [[../iterations/iteration-23-hyalo-best-practices]]
