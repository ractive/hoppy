# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-05-09

### Release pipeline (iter-25)

- SHA-pinned GitHub Actions in `ci.yml` and `release.yml`
- `version-check` job: release tag must match `Cargo.toml` version
- `security` job: `cargo audit` + `cargo deny check` before any build
- `crates-io` job: publishes all 9 crates in dependency order with index-propagation retry
- `scoop` job: updates `ractive/scoop-bucket` Homebrew manifest on each release
- Homebrew formula updated with macOS x86 target, caveats for optional `bore` dependency
- `deny.toml` added for license and advisory gating
- All crate `Cargo.toml` files: added `homepage`, `keywords`, `categories`, `readme`; internal workspace deps now carry `version = "0.1.0"` for crates.io compatibility
- Minimal `README.md` added to each sub-crate

### Magic Containers — container logs tunnel (iter-24)

- `hoppy container logs` — streams Magic Containers syslog via an in-process RFC 5424/3164 receiver
- Auto-tunnel via [bore](https://github.com/ekzhang/bore): zero-config public ingress on a kernel-assigned port
- `--tunnel none` / `--tunnel-host host:port` for custom ingress (SSH reverse-forward or private bore server)
- `--bore-server` flag to point at a self-hosted bore relay
- `bunny-syslog-receiver` crate extracted as a reusable, independently publishable library
- NDJSON output with `--format json`; `--format table` rejected (logs are not tabular)

### Project hygiene & dogfooding (iter-23)

- Cargo workspace hygiene: `[workspace.dependencies]`, `[workspace.package]`, `resolver = "3"`, optimised `[profile.release]`, `unsafe_code = "forbid"`
- `AI_NOTICE`, dogfooding playbook, CLI command tree, help-text style guide

### Stream — video processing (iter-16)

- `stream video transcribe`, `heatmap`, `reencode`, `repackage`, `smart-generate`, `set-thumbnail`, `resolutions list/cleanup`, `storage`

### Magic Containers — UX & safety (iter-21)

- Granular env-var mutation (`--add`, `--remove`, `--update`, `--replace-all`, `--clear`, `--list`)
- `container app delete --cascade` / `--no-cascade` to control Pull Zone orphaning
- `container app create` returns full document by default; `--minimal` for legacy shape
- `container app create --env KEY=VAL` for initial env in one call
- Secret redaction: env values masked as `<set, length=N>`; `--reveal` / `--reveal-env KEY` to opt in
- Confirmation phrases required for destructive env operations

## [0.1.0] - 2026-03-18

Initial release.

### Services

- **CDN Pull Zones** — list, get, create, update, delete, purge cache (by tag or full)
- **Storage Zones** — list, get, create, update, delete
- **Storage Files** — upload, download, list, delete with progress bars
- **DNS** — zone CRUD, record management for all record types (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA, and bunny-specific types)
- **Video Streaming** — library CRUD, video list/get/upload/delete with progress bars
- **Shield (Security)** — zone management, WAF rules, rate limiting, access lists, bot detection, DDoS configuration
- **Edge Scripting** — script CRUD, publish, code get/update, releases, variables, secrets, statistics
- **Magic Containers** — applications, templates, endpoints, volumes, registries, regions, nodes, pods, limits, log forwarding
- **Auth** — API key validation with billing/account info

### Features

- Three output formats: `--format json|table|text`
- Debug mode (`--debug`) showing HTTP request details
- Quiet mode (`--quiet`) suppressing non-essential output
- Confirmation prompts for destructive operations (`--yes` to skip)
- Progress bars for file uploads and downloads
- Shell completions for bash, zsh, and fish
- Pagination support across all list commands
- Credentials excluded from JSON output for security

[Unreleased]: https://github.com/ractive/hoppy/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/ractive/hoppy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ractive/hoppy/releases/tag/v0.1.0
