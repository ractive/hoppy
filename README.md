# hoppy

[![CI](https://github.com/ractive/hoppy/actions/workflows/ci.yml/badge.svg)](https://github.com/ractive/hoppy/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A CLI for [bunny.net](https://bunny.net) cloud and edge services. Designed for both humans and AI agents.

## Features

| Service | Commands |
|---------|----------|
| **CDN Pull Zones** | list, get, create, update, delete, purge cache |
| **Storage Zones** | list, get, create, update, delete |
| **Storage Files** | upload, download, list, delete (with progress bars) |
| **DNS** | zone CRUD, record management for all types (A, AAAA, CNAME, MX, TXT, SRV, CAA, ...) |
| **Video Streaming** | library CRUD, video list/get/upload/delete (with progress bars) |
| **Shield (Security)** | WAF rules, rate limiting, access lists, bot detection, DDoS config |
| **Edge Scripting** | script CRUD, publish, code management, variables, secrets, statistics |
| **Magic Containers** | apps, templates, endpoints, volumes, registries, regions, nodes, pods |
| **Auth** | API key validation, billing/account info |

## Installation

### Homebrew (macOS / Linux)

```bash
brew install ractive/hoppy/hoppy
```

### Direct download

Download the latest binary from [GitHub Releases](https://github.com/ractive/hoppy/releases/latest).

Archives include the binary, shell completions, man page, and LICENSE.

### From source

```bash
cargo install --git https://github.com/ractive/hoppy
```

Or clone and build:

```bash
git clone https://github.com/ractive/hoppy.git
cd hoppy
cargo build --release
# Binary is at target/release/hoppy
```

### Linux packages

`.deb` and `.rpm` packages are available on the [releases page](https://github.com/ractive/hoppy/releases/latest). They include shell completions and man pages.

```bash
# Debian / Ubuntu
sudo dpkg -i hoppy_0.1.0_amd64.deb

# Fedora / RHEL
sudo rpm -i hoppy-0.1.0-1.x86_64.rpm
```

### Windows

Download the `.zip` from [GitHub Releases](https://github.com/ractive/hoppy/releases/latest) and add the binary to your PATH.

Also available via [winget](https://github.com/microsoft/winget-pkgs):

```powershell
winget install ractive.hoppy
```

## Quick start

```bash
export BUNNY_API_KEY=your-api-key
hoppy auth check
```

## Usage

### CDN Pull Zones

```bash
hoppy pull-zone list
hoppy pull-zone get --id 123456
hoppy pull-zone create --name my-zone --origin-url https://origin.example.com
hoppy pull-zone update --id 123456 --origin-url https://new-origin.example.com
hoppy pull-zone delete --id 123456 --yes
hoppy pull-zone purge --id 123456
hoppy pull-zone purge --id 123456 --cache-tag static-assets
```

### Storage

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

### DNS

```bash
hoppy dns zone list
hoppy dns zone create --domain example.com
hoppy dns record list --zone-id 123
hoppy dns record add --zone-id 123 --type A --name www --value 1.2.3.4
hoppy dns record add --zone-id 123 --type MX --name @ --value mail.example.com --priority 10
hoppy dns record update --zone-id 123 --record-id 456 --value 5.6.7.8
hoppy dns record delete --zone-id 123 --record-id 456 --yes
```

### Video Streaming

```bash
hoppy stream library list
hoppy stream video list --library-id 456
hoppy stream video upload --library-id 456 --file ./video.mp4
hoppy stream video get --library-id 456 --video-id abc-123
hoppy stream video delete --library-id 456 --video-id abc-123 --yes
```

### Shield (Security)

```bash
hoppy shield zone list
hoppy shield waf list-rules --shield-zone-id 789
hoppy shield waf add-rule --shield-zone-id 789 --rule-type custom --action-type block
hoppy shield rate-limit list --shield-zone-id 789
hoppy shield access-list list --shield-zone-id 789
hoppy shield bot-detection get --shield-zone-id 789
```

### Edge Scripting

```bash
hoppy script list
hoppy script code get --id 123
hoppy script code update --id 123 --file ./handler.js
hoppy script publish --id 123
hoppy script variable list --id 123
hoppy script secret list --id 123
hoppy script statistics --id 123 --date-from 2026-01-01 --date-to 2026-01-31
```

### Magic Containers

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

## Global options

| Flag | Description |
|------|-------------|
| `--format json\|table\|text` | Output format (default: table) |
| `--debug` | Show HTTP request details |
| `--quiet` | Suppress non-essential output |
| `--yes` / `-y` | Skip confirmation prompts |

## Environment variables

| Variable | Description |
|----------|-------------|
| `BUNNY_API_KEY` | **Required.** Your bunny.net API key. |
| `BUNNY_STORAGE_KEY` | Storage zone API key (optional — auto-resolved from zone details if not set). |
| `BUNNY_STREAM_KEY` | Stream library API key (optional — auto-resolved from library details if not set). |

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

## AI-generated project

This project is largely AI-generated ("vibe coded"). See [LICENSE](LICENSE) for details on copyright and usage. Users are responsible for ensuring compliance with any third-party rights that may apply.

## License

MIT
