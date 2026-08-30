# hoppy manual

The README is intentionally lean. This page collects the exhaustive surface:
every service area with worked example invocations, global flags,
environment variables, and testing setup. Use `hoppy <command> --help` for
authoritative per-command help — anything here is example-shaped.

For a one-line tree of every subcommand, see
[`hoppy-knowledgebase/cli/command-tree.md`](../hoppy-knowledgebase/cli/command-tree.md).

## Features at a glance

| Service | Commands |
|---------|----------|
| **CDN Pull Zones** | list, get, create, update, delete, purge cache |
| **Storage Zones** | list, get, create, update, delete |
| **Storage Files** | upload, download, list, delete (with progress bars) |
| **DNS** | zone CRUD, record management (A, AAAA, CNAME, MX, TXT, SRV, CAA, ...), DNSSEC, wildcard cert issuance, record discovery scans |
| **Video Streaming** | library CRUD, video list/get/upload/delete (with progress bars), transcription, re-encoding, repackage, smart-generate, thumbnails, heatmaps, resolution management, storage breakdown |
| **Shield (Security)** | WAF rules, rate limiting, access lists, bot detection, DDoS config |
| **Edge Scripting** | script CRUD, publish, code management, variables, secrets, statistics |
| **Magic Containers** | apps, templates, endpoints, volumes, registries, regions, nodes, pods |
| **Database (libSQL)** | databases, groups, tokens, ping (data plane), config, statistics |
| **Auth** | API key validation, billing/account info |
| **Account & Billing** | API keys, billing summary, payment requests, invoice PDFs, region/country reference data, global search, audit log |

## CDN Pull Zones

```bash
hoppy pull-zone list
hoppy pull-zone get --id 123456
hoppy pull-zone create --name my-zone --origin-url https://origin.example.com
hoppy pull-zone update --id 123456 --origin-url https://new-origin.example.com
hoppy pull-zone delete --id 123456 --yes
hoppy pull-zone purge --id 123456
hoppy pull-zone purge --id 123456 --cache-tag static-assets

# Referrer access control (anti-hotlinking)
hoppy pull-zone referrer list --id 123456
hoppy pull-zone referrer allow --id 123456 --value '*.example.com'
hoppy pull-zone referrer block --id 123456 --value badsite.com
hoppy pull-zone referrer remove-allowed --id 123456 --value '*.example.com'
hoppy pull-zone referrer remove-blocked --id 123456 --value badsite.com

# IP block list
hoppy pull-zone ip list --id 123456
hoppy pull-zone ip block --id 123456 --value 192.0.2.1
hoppy pull-zone ip unblock --id 123456 --value 192.0.2.1
```

## Storage

```bash
# Manage storage zones
hoppy storage-zone list
hoppy storage-zone create --name my-storage --region DE

# File operations
hoppy storage ls --zone my-zone --path /
hoppy storage upload --zone my-zone --remote-path /file.txt --file ./local.txt
hoppy storage download --zone my-zone --remote-path /file.txt --output ./local.txt
hoppy storage rm --zone my-zone --remote-path /file.txt --yes
```

## DNS

```bash
hoppy dns zone list
hoppy dns zone create --domain example.com
hoppy dns record list --zone-id 123
hoppy dns record add --zone-id 123 --type A --name www --value 1.2.3.4
hoppy dns record add --zone-id 123 --type MX --name @ --value mail.example.com --priority 10
hoppy dns record update --zone-id 123 --record-id 456 --value 5.6.7.8
hoppy dns record delete --zone-id 123 --record-id 456 --yes

# DNSSEC
hoppy dns zone dnssec enable --id 123        # prints DS record to copy to your registrar
hoppy dns zone dnssec status --id 123
hoppy dns zone dnssec disable --id 123 --yes # warns: remove DS records at registrar first

# Wildcard certificate (zone must be delegated to bunny.net nameservers)
hoppy dns zone issue-cert --id 123

# Record discovery scan (auto-discover records during migration)
hoppy dns zone scan start --id 123              # or: --domain example.com
hoppy dns zone scan results --id 123
```

## Video Streaming

```bash
hoppy stream library list
hoppy stream video list --library-id 456
hoppy stream video upload --library-id 456 --file ./video.mp4
hoppy stream video get --library-id 456 --video-id abc-123
hoppy stream video delete --library-id 456 --video-id abc-123 --yes

# Processing & analytics
hoppy stream video transcribe --library-id 456 --video-id abc-123
hoppy stream video heatmap --library-id 456 --video-id abc-123
hoppy stream video reencode --library-id 456 --video-id abc-123
hoppy stream video reencode --library-id 456 --video-id abc-123 --codec hevc
hoppy stream video repackage --library-id 456 --video-id abc-123
hoppy stream video smart-generate --library-id 456 --video-id abc-123 --generate-title
hoppy stream video set-thumbnail --library-id 456 --video-id abc-123 --thumbnail-url https://...
hoppy stream video resolutions list --library-id 456 --video-id abc-123
hoppy stream video resolutions cleanup --library-id 456 --video-id abc-123 --dry-run
hoppy stream video storage --library-id 456 --video-id abc-123
```

## Shield (Security)

```bash
hoppy shield zone list
hoppy shield waf list-rules --shield-zone-id 789
hoppy shield waf add-rule --shield-zone-id 789 --rule-type custom --action-type block
hoppy shield rate-limit list --shield-zone-id 789
hoppy shield access-list list --shield-zone-id 789
hoppy shield bot-detection get --shield-zone-id 789
```

## Edge Scripting

```bash
hoppy script list
hoppy script code get --id 123
hoppy script code update --id 123 --file ./handler.js
hoppy script publish --id 123
hoppy script variable list --id 123
hoppy script secret list --id 123
hoppy script statistics --id 123 --date-from 2026-01-01 --date-to 2026-01-31
```

## Magic Containers

```bash
hoppy container app list
hoppy container app get --id app-uuid
hoppy container app create --name my-app --runtime-type Shared --min 1 --max 3 --region DE
hoppy container template get --app-id app-uuid --container-id ctr-uuid
hoppy container endpoint list --app-id app-uuid
hoppy container registry list
hoppy container region list
hoppy container limits
```

### Tailing Magic Containers logs

> **Token is required (undocumented upstream):** `POST /mc/log/forwarding`
> rejects any configuration without a `token` with an empty HTTP 400, even
> though Bunny's spec marks the field optional. hoppy handles this for
> you: `container logs` auto-generates a session token, and
> `container log-forwarding create`/`update` make `--token` a required
> flag. (This empty 400 masqueraded as a broken endpoint from 2026-05-15
> until the root cause was found on 2026-08-09.)

Bunny does not expose a logs-fetch API for Magic Containers — logs are syslog-forwarded only, so there's no `--tail` flag to look for.

```bash
hoppy container logs --app-id app-uuid
```

This spins up a local TCP syslog (RFC 5424) receiver on a kernel-assigned port, opens a public tunnel via [bore](https://github.com/ekzhang/bore), registers a Bunny log-forwarding configuration pointed at the tunnel's public address, and streams incoming lines to your terminal. Ctrl-C tears everything down (forwarding config, tunnel, receiver).

If you see `bore: command not found`:

```bash
cargo install bore-cli      # or
brew install bore-cli
```

If you already have public ingress, skip bore entirely:

```bash
hoppy container logs --app-id app-uuid --tunnel none
hoppy container logs --app-id app-uuid --tunnel-host vps.example.com:5514   # after `ssh -R 5514:localhost:5514 user@vps`
```

Privacy note: `bore.pub` is a third-party relay run by the bore project — your log lines traverse it. For sensitive workloads use `--tunnel-host` with your own ingress, or run `bore server` on your own VPS and pass `--bore-server <host>`.

Bunny adds a 10–30 second delivery delay before forwarded lines arrive, so don't expect real-time tailing.

JSON output: pass `--format json` for NDJSON. `--format table` is rejected (logs aren't tabular).

## Database (libSQL)

```bash
# Group + DB lifecycle
hoppy db group create --display-name EU --storage-region eu-west-1 \
    --primary-region DE --replicas-region UK
hoppy db create --slug my-app --group group_01HX...
hoppy db list
hoppy db get --id db_01HX...

# Auth tokens (JWT redacted by default — pass --reveal to print)
hoppy db token mint --db-id db_01HX... --authorization full-access
hoppy db token invalidate --db-id db_01HX...

# Data-plane health check (mints a short-lived read-only token automatically)
hoppy db ping --id db_01HX...

# Config & live metrics
hoppy db config show
hoppy db live --id db_01HX... --id db_02HX...
```

By default, `hoppy db token mint` prints `{ "token": "<set, length=N>", ... }`
to keep JWTs out of logs and CI output. Use the global `--reveal` flag to opt
in to the raw token. Slugs are validated locally (`^[a-z][a-z0-9-]{0,23}$`)
because the bunny API silently 500s on overlong values.

## Account & Billing

```bash
# API keys — values are redacted by default (pass --reveal to show them)
hoppy apikey list
hoppy apikey list --reveal

# Billing summary + payment requests
hoppy billing summary
hoppy billing payment-requests

# Invoice PDFs (streamed straight to disk, never buffered whole)
hoppy billing invoice-pdf --record-id 44001 --output invoice.pdf
hoppy billing payment-request-pdf --id 90002 --output payment-request.pdf

# Reference data
hoppy region list          # core CDN edge regions + per-GB pricing
hoppy country list         # ISO codes valid for pull-zone country blocking

# Global cross-resource search
hoppy search example --size 20

# Account audit log for a date (ISO 8601)
hoppy user audit --date 2026-07-01
hoppy user audit --date 2026-07-01 --resource-type PullZone --order descending
```

API-key values are redacted (`<set, length=N>`) in table, text, **and** JSON
output unless `--reveal` is passed, so a `--format json | jq` pipeline never
leaks a key into a logfile. The two `billing *-pdf` downloads stream the
response body chunk-by-chunk to the `--output` file. `user audit` paginates
via a continuation token surfaced in a stderr hint (or the JSON
`ContinuationToken` field) — pass it back with `--continuation-token`.

## Global options

| Flag | Description |
|------|-------------|
| `--format json\|table\|text` | Output format (default: table) |
| `--debug` | Show HTTP request details |
| `--quiet` | Suppress non-essential output |
| `--yes` / `-y` | Skip confirmation prompts |
| `--reveal` | Print raw secrets (tokens, passwords, env values) instead of redacting them |
| `--reveal-env KEY` | Reveal a specific env-var by name (repeatable) |
| `--no-hints` | Suppress drill-down hint lines on stderr (implied by `--format json`) |

### JSON output shapes

`--format json` passes each bunny.net API's own serialization through
unchanged, so wrapper keys and field casing differ by service family:

| Surface | Wrapper key | Field casing | Pagination meta |
|---------|-------------|--------------|-----------------|
| `pull-zone`, `storage-zone`, `dns`, `script`, `stream` | `Items` | PascalCase (`Id`, `Name`; dns uses `Domain`) | `CurrentPage`, `TotalItems`, `HasMoreItems` |
| `statistics` | top-level object | PascalCase | none |
| `shield` | `Items` | camelCase (`shieldZoneId`, `wafEnabled`) | `CurrentPage`, `TotalItems`, `HasMoreItems` |
| `container`, `db` | `items` | camelCase (`id`, `name`) | `cursor`, `meta` |

Scripts consuming more than one surface need per-surface `jq` paths (e.g.
`.Items[].Name` for pull zones but `.items[].name` for container apps).
Normalized output is a possible future feature; until then this table is
the contract.

## Environment variables

| Variable | Description |
|----------|-------------|
| `BUNNY_API_KEY` | **Required.** Your bunny.net API key. |
| `BUNNY_STORAGE_KEY` | Storage zone API key (optional — auto-resolved from zone details if not set). |
| `BUNNY_STREAM_KEY` | Stream library API key (optional — auto-resolved from library details if not set). |

## Testing

### Mock tests (default)

All CLI tests use [wiremock](https://crates.io/crates/wiremock) to mock the Bunny API, so they run instantly without network access:

```bash
cargo test
```

### Live API tests

Tests can also run against the real bunny.net API using the `live-api` Cargo feature. This requires a `BUNNY_API_KEY` environment variable set to a valid API key.

> **Warning:** Use a dedicated test/sandbox account — live tests may create, modify, and delete real resources.

```bash
export BUNNY_API_KEY=your-test-api-key
cargo test --features live-api
```

The `live-api` feature enables additional test helpers:

| Helper | Purpose |
|--------|---------|
| `hoppy_live_cmd()` | Builds the CLI binary with your real `BUNNY_API_KEY` |
| `unique_name(prefix)` | Generates collision-free resource names (e.g. `my-zone-1710849600000-0`) |

### Environment variable overrides for testing

You can point any API client at a custom endpoint (useful for proxies, staging, or local mocks):

| Variable | Default |
|----------|---------|
| `BUNNY_API_URL` | `https://api.bunny.net` (core, compute, shield) |
| `BUNNY_STORAGE_URL` | `https://{region}.storage.bunnycdn.com` |
| `BUNNY_STREAM_URL` | `https://video.bunnycdn.com` |
| `BUNNY_CONTAINERS_URL` | `https://api.bunny.net` |
| `BUNNY_DATABASE_URL` | `https://api.bunny.net/database` |

## Shell completions

```bash
# Bash
hoppy completions bash > ~/.local/share/bash-completion/completions/hoppy

# Zsh (add ~/.zfunc to fpath in .zshrc before compinit)
hoppy completions zsh > ~/.zfunc/_hoppy

# Fish
hoppy completions fish > ~/.config/fish/completions/hoppy.fish
```

Package installs (Homebrew, deb, rpm) include completions automatically.

## Version provenance

`hoppy -V` prints the package version, short git SHA, and commit date the
binary was built from — for example:

```text
$ hoppy -V
hoppy 0.3.0 (abc123def456 2026-05-26)
```

When the working tree had uncommitted changes at build time, the SHA is
suffixed `+dirty`. Builds from a crates.io tarball (`cargo install hoppy-cli`)
have no `.git` tree but carry the publishing commit in `.cargo_vcs_info.json`,
so they print the SHA without a date: `hoppy 0.3.0 (abc123def456)`. Other
tarball builds without either fall back to just `hoppy 0.3.0`; CI can
override either field via the `GIT_COMMIT` and `GIT_COMMIT_DATE` build-time
env vars, or force the no-git path with `CARGO_HOPPY_FORCE_NO_GIT=1`.
