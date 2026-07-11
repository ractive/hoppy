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

Many commands suggest a sensible next step on stderr (turn off with
`--no-hints`), so you can drill in without leaving the terminal.

## Install

**Homebrew** (macOS arm64, Linux x86_64/arm64 — Linux gets static musl binaries):

```bash
brew install ractive/tap/hoppy
```

**Debian / Ubuntu** (apt) — from the hosted [Cloudsmith](https://cloudsmith.io) repo:

```bash
curl -sLf 'https://dl.cloudsmith.io/public/ractive/hoppy/cfg/setup/bash.deb.sh' | sudo bash
sudo apt install hoppy
```

**Fedora / RHEL / openSUSE** (dnf/yum/zypper):

```bash
curl -sLf 'https://dl.cloudsmith.io/public/ractive/hoppy/cfg/setup/bash.rpm.sh' | sudo bash
sudo dnf install hoppy   # or: yum install hoppy / zypper install hoppy
```

**Windows** (Scoop):

```bash
scoop bucket add ractive https://github.com/ractive/scoop-bucket
scoop install hoppy
```

**Anywhere with Rust** (Cargo):

```bash
cargo install hoppy-cli   # binary is `hoppy`
```

The apt/dnf packages and the `.deb`/`.rpm` files also install shell
completions and man pages system-wide. Standalone `.deb`/`.rpm` files and
prebuilt archives for every target — including static musl Linux builds, each
bundling the binary, shell completions, and man pages — are on the
[releases page](https://github.com/ractive/hoppy/releases/latest). Windows/winget
support is not available yet.

### Shell completions

The packaged installs wire completions up for you. For a manual setup (e.g.
the `cargo install` route), generate them with:

```bash
hoppy completions bash > ~/.local/share/bash-completion/completions/hoppy
hoppy completions zsh  > "${fpath[1]}/_hoppy"
hoppy completions fish > ~/.config/fish/completions/hoppy.fish
```

### Verifying downloads

Release archives for native targets ship [CycloneDX](https://cyclonedx.org)
SBOMs and GitHub build-provenance attestations. Verify an archive with the
GitHub CLI:

```bash
gh attestation verify hoppy-v0.5.0-aarch64-apple-darwin.tar.gz --owner ractive
```

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

## Package repository hosting

[![OSS hosting by Cloudsmith](https://img.shields.io/badge/OSS%20hosting%20by-cloudsmith-blue?logo=cloudsmith&style=flat-square)](https://cloudsmith.com)

Package repository hosting is graciously provided by [Cloudsmith](https://cloudsmith.com).
Cloudsmith is the only fully hosted, cloud-native, universal package management solution, that
enables your organization to create, store and share packages in any format, to any place, with total
confidence.

## License

MIT
