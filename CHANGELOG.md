# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Stream — video processing (iter-16)

New commands for triggering and inspecting video processing on bunny.net Stream:

- `stream video transcribe` — trigger transcription/translation, with optional source/target languages and AI generation flags
- `stream video heatmap` — fetch the viewer engagement heatmap for a video
- `stream video reencode` — re-encode a video (optionally targeting a specific codec: `x264`, `vp9`, `hevc`, `av1`)
- `stream video repackage` — repackage HLS/DASH manifests (optionally discarding previous file versions)
- `stream video smart-generate` — trigger AI title/description/chapters/moments generation
- `stream video set-thumbnail` — set the video thumbnail from a URL
- `stream video resolutions list` — show available/configured/playlist resolutions
- `stream video resolutions cleanup` — delete renditions (with dry-run and confirmation gate)
- `stream video storage` — show per-category storage size breakdown (originals, thumbnails, previews, MP4 fallback, per-rendition encoded sizes)

### Magic Containers — UX & safety (iter-21)

- **`container template env` no longer silently wipes env vars.** A bare
  invocation now errors with a recipe. New flags: `--add KEY=VAL`,
  `--remove KEY`, `--update KEY=VAL` (granular merge), `--replace-all
  --env K=V …` (explicit destructive replace), `--clear` (explicit wipe),
  `--list` (show current env, redacted by default).
- **`container app delete` now refuses to orphan auto-managed Pull Zones.**
  Pass `--cascade` to delete the app + its CDN Pull Zones, or `--no-cascade`
  to delete only the app and print orphan IDs with a cleanup recipe.
- **`container app create` returns the full app document by default.**
  No more chained `app get` calls to grab template / endpoint ids. Pass
  `--minimal` to opt back into the legacy `{"id": "..."}` shape.
- **`container app create --env KEY=VAL`** sets initial env vars in one call
  (combine with the image flags).
- **Secret redaction.** Env-var values are masked as `<set, length=N>` (or
  `<unset>`) in JSON, table, and text output. Opt in with the global
  `--reveal` (all secrets) or `--reveal-env KEY` (a specific var).
- Destructive `--clear` and shrinking `--replace-all` now require typing
  "wipe" / "replace" — `--yes` alone is not enough.

## [0.1.0] — 2026-03-18

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

[0.1.0]: https://github.com/ractive/hoppy/releases/tag/v0.1.0
