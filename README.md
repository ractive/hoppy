# hoppy

[![CI](https://github.com/ractive/hoppy/actions/workflows/ci.yml/badge.svg)](https://github.com/ractive/hoppy/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A friendly CLI for [bunny.net](https://bunny.net) — CDN, storage, DNS,
streaming, edge compute, magic containers, and libSQL — designed for humans
and AI agents alike.

```bash
export BUNNY_API_KEY=your-api-key
hoppy pull-zone list
```

Every command suggests a sensible next step on stderr (turn off with
`--no-hints`), so you can drill in without leaving the terminal.

## Install

```bash
# macOS / Linux
brew tap ractive/tap && brew install hoppy

# Windows
scoop bucket add ractive https://github.com/ractive/scoop-bucket
scoop install hoppy

# Anywhere with Rust
cargo install hoppy-cli   # binary is `hoppy`
```

`.deb` / `.rpm` archives (with shell completions + man pages) and prebuilt
binaries are on the [releases page](https://github.com/ractive/hoppy/releases/latest).

## Quick start

```bash
export BUNNY_API_KEY=your-api-key

hoppy auth check
hoppy pull-zone list
hoppy pull-zone create --name my-zone --origin-url https://origin.example.com
hoppy stream library list
hoppy container app list
```

## Where to go next

- [`docs/MANUAL.md`](docs/MANUAL.md) — examples for every service area,
  global flags, env vars, testing setup.
- [`hoppy-knowledgebase/cli/command-tree.md`](hoppy-knowledgebase/cli/command-tree.md)
  — the full subcommand tree at a glance.
- `hoppy <command> --help` — authoritative per-flag help.
- [dash.bunny.net](https://dash.bunny.net) — bunny.net concept docs.

## AI-generated project

This project is largely AI-generated ("vibe coded"). See [LICENSE](LICENSE) for
details on copyright and usage. Users are responsible for ensuring compliance
with any third-party rights that may apply.

## License

MIT
