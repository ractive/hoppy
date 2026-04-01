---
title: CLI Design Patterns from Cloud CLIs (az, gcloud, aws)
date: 2026-03-17
tags:
  - cli
  - design-patterns
  - azure
  - gcloud
  - aws
  - ux
status: research-complete
type: research
---

# CLI Design Patterns from Cloud CLIs

## Command Structure Comparison

### Azure CLI (az)
```
az <group> <subgroup> <command> [--parameters]
az vm create --resource-group myGroup --name myVM --image Ubuntu2204
az storage blob list --account-name myaccount --container-name mycontainer
```
- Groups/subgroups map to Azure resource types
- Kebab-case for multi-word groups: `az container-app`, `az cdn endpoint`

### Google Cloud CLI (gcloud)
```
gcloud <group> <subgroup> <command> [--parameters]
gcloud compute instances create my-instance --zone=us-central1-a
gcloud dns managed-zones list
```
- Similar hierarchical structure
- Uses `=` for parameter values: `--zone=us-central1-a`

### AWS CLI
```
aws <service> <command> [--parameters]
aws s3 ls
aws ec2 describe-instances --instance-ids i-1234567890abcdef0
aws route53 list-hosted-zones
```
- Flatter structure (service + action, no subgroups usually)
- Verb-noun naming: `describe-instances`, `create-bucket`

## Recommended Structure for Hoppy

Based on the patterns above, the most intuitive structure:

```
hoppy <service> <action> [--parameters]
```

Examples:
```bash
hoppy pull-zone list
hoppy pull-zone get --id 123456
hoppy pull-zone create --name "my-zone" --origin-url "https://origin.example.com"
hoppy pull-zone delete --id 123456
hoppy pull-zone purge --id 123456 --urls "https://example.com/path"

hoppy storage-zone list
hoppy storage upload --zone "my-zone" --path "/remote/file.txt" --file "./local/file.txt"
hoppy storage ls --zone "my-zone" --path "/"

hoppy dns zone list
hoppy dns record add --zone-id 123 --type A --name "www" --value "1.2.3.4"

hoppy stream video upload --library-id 456 --file ./video.mp4
hoppy stream video list --library-id 456
```

## Output Formatting

### Industry Standard

All major CLIs support `--output` or `-o` with these formats:

| CLI | Flag | Formats | Default |
|-----|------|---------|---------|
| az | `--output`, `-o` | json, table, tsv, yaml, yamlc, jsonc, none | json |
| gcloud | `--format` | json, yaml, table, csv, text, config, value | table-like |
| aws | `--output` | json, table, text, yaml | json |

### Recommendations for Hoppy

- Use `--format` flag (matches gcloud, avoids confusion with `-o` for other things)
- Support: `json`, `table`, `text`
- Default to `table` for humans, `json` for machine consumption
- Consider auto-detecting: if stdout is a TTY, default to table; if piped, default to json
- Also settable via `HOPPY_OUTPUT_FORMAT` env var

## Authentication Patterns

### How Major CLIs Do It

1. **Environment variable**: `AWS_ACCESS_KEY_ID`, `AZURE_CLIENT_ID`
2. **Config file**: `~/.aws/credentials`, `~/.azure/`
3. **Login command**: `az login`, `gcloud auth login`
4. **Per-command flag**: `--access-key`, `--project`

### Recommendations for Hoppy

Priority order (first found wins):
1. `--api-key` command-line flag
2. `BUNNY_API_KEY` environment variable
3. Config file at `~/.config/hoppy/config.toml` (future)
4. `hoppy auth login` command (future - stores key in config)

## Pagination

### Industry Patterns

- aws: `--max-items`, `--page-size`, `--starting-token`; automatic pagination with `--no-paginate` to disable
- az: `--top` for page size
- gcloud: `--page-size`, `--limit`

### Recommendations for Hoppy

- `--page` and `--per-page` for manual pagination
- `--all` flag to auto-paginate and return all results
- Default: show first page (reasonable default like 25-50 items)

## AI/LLM-Friendly Patterns

To make the CLI useful for AI agents:

1. **Structured JSON output** - Always support `--format json` with consistent schema
2. **Predictable command patterns** - Consistent CRUD verbs: `list`, `get`, `create`, `update`, `delete`
3. **Exit codes** - 0 for success, non-zero for errors with JSON error output
4. **Self-describing** - `hoppy --help`, `hoppy pull-zone --help`, `hoppy pull-zone create --help`
5. **No interactive prompts** - All input via flags/args, never prompt for input (or use `--yes` to skip confirmation)
6. **Idempotent operations** where possible
7. **Error output in JSON** when `--format json` is set:
   ```json
   {"error": {"code": "NOT_FOUND", "message": "Pull zone 123 not found"}}
   ```

## Additional UX Patterns

### Async Operations
- For long-running operations, print a status and return immediately
- Support `--wait` flag to block until complete
- Default: return immediately with operation ID

### Filtering/Querying
- Consider `--query` flag with JMESPath (like az/aws) for filtering JSON output
- Or simpler `--filter` with key=value matching

### Confirmation for Destructive Actions
- `delete` commands should require `--yes` or `-y` to skip confirmation
- When stdout is not a TTY, require `--yes` (no interactive prompt possible)

### Verbose/Debug Mode
- `--verbose` or `-v` for detailed output
- `--debug` for HTTP request/response logging (useful for API debugging)

## clig.dev — Command Line Interface Guidelines

The [CLI Guidelines](https://clig.dev/) is a comprehensive, community-maintained guide. Key takeaways for hoppy:

### Core Principles
- **Human-first design** — the CLI is a text-based UI for humans, not just a scripting interface
- **Composability** — work well with pipes, stdout/stderr, exit codes, plain text or JSON
- **Consistency** — follow established flag/argument conventions; terminal habits are muscle memory
- **Robustness** — handle unexpected input gracefully; output something within 100ms

### Output Rules
- Primary output to **stdout**, logs/errors/status to **stderr**
- Detect TTY: human-readable tables for interactive use, JSON for pipes
- Respect `NO_COLOR` env var and `TERM=dumb`
- Explain what changed on success; suggest next commands
- Show progress indicators for long operations

### Error Handling
- Rewrite errors for humans with actionable guidance
- Place most important information at end of output
- Don't print stack traces by default
- Group similar errors under explanatory headers

### Arguments & Flags
- **Prefer flags to positional arguments** for clarity
- Provide both short (`-h`) and long (`--help`) forms
- Standard flag names to follow: `--all`, `--debug`, `--force`, `--json`, `--dry-run`, `--no-input`, `--quiet`, `--version`
- **Never read secrets via flags** (visible in ps/history) — use env vars, files, or stdin
- Make defaults right for most users
- Allow order-independent flags

### Interactivity
- Only prompt when stdin is a TTY
- Respect `--no-input` / `--yes` flags to disable all prompts
- Confirm before dangerous actions (delete, overwrite)
- Ctrl-C must always work

### Configuration Precedence (highest to lowest)
1. Command-line flags
2. Environment variables
3. Project-level config
4. User-level config
5. System-wide config

### Environment Variables
- Uppercase with underscores: `BUNNY_API_KEY`, `HOPPY_OUTPUT_FORMAT`
- Check standard vars: `NO_COLOR`, `DEBUG`, `HTTP_PROXY`, `TERM`, `PAGER`
- Read `.env` files when appropriate
- Note: clig.dev warns against storing secrets in env vars (prone to leakage via logs/child processes) — but for API keys this is the industry standard approach (az, aws, gcloud all do it)

### Distribution
- Distribute as a **single binary** (perfect fit for Rust)
- Make uninstallation easy
- No phoning home without explicit consent

### Naming
- Simple, memorable, lowercase, short
- "hoppy" fits well: short, memorable, easy to type

### Implications for Hoppy

| Guideline | Hoppy Implementation |
|-----------|---------------------|
| Human-first output | Default `--format table` for TTY, `--format json` for pipes |
| Stderr for status | Progress, warnings, auth errors go to stderr |
| Standard flags | `--help`, `--version`, `--format`, `--debug`, `--quiet`, `--yes`, `--dry-run` |
| No secret flags | `BUNNY_API_KEY` env var (not `--api-key` flag) |
| Config precedence | flags > env vars > `~/.config/hoppy/config.toml` |
| Single binary | Rust cross-compiled binaries via GitHub Actions |
| Confirm destructive ops | `delete` commands require `--yes` or interactive confirmation |
| Crash-only design | Exit immediately on failure, don't leave half-done state |

## Sources

- [AWS CLI Output Formats](https://docs.aws.amazon.com/cli/latest/userguide/cli-usage-output-format.html)
- [Azure CLI Output Formats](https://learn.microsoft.com/en-us/cli/azure/format-output-azure-cli)
- [Command Line Interface Guidelines](https://clig.dev/)
- [Make Your CLI a Joy to Use](https://www.caduh.com/blog/make-your-cli-a-joy-to-use)

## Related
- [[research/rust-cli-best-practices]] — Rust-specific CLI best practices
- [[Seed]] — project brief with CLI design goals
- [[decision-log]] — CLI-related decisions
