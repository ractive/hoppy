# hoppy

A CLI for [bunny.net](https://bunny.net) cloud and edge services. Designed for both humans and AI agents.

## Quick start

```bash
export BUNNY_API_KEY=your-api-key

# CDN pull zones
hoppy pull-zone list
hoppy pull-zone get --id 123456
hoppy pull-zone create --name my-zone --origin-url https://origin.example.com
hoppy pull-zone delete --id 123456 --yes

# Storage
hoppy storage-zone list
hoppy storage ls --zone my-zone --path /
hoppy storage upload --zone my-zone --remote-path /file.txt --file ./local.txt

# DNS
hoppy dns zone list
hoppy dns record add --zone-id 123 --type A --name www --value 1.2.3.4

# Video streaming
hoppy stream video list --library-id 456
hoppy stream video upload --library-id 456 --file ./video.mp4

# Shield (WAF / security)
hoppy shield zone list
hoppy shield waf list-rules --shield-zone-id 789
hoppy shield rate-limit list --shield-zone-id 789

# Edge scripting
hoppy script list
hoppy script code get --id 123
hoppy script publish --id 123

# Output formats
hoppy pull-zone list --format json
hoppy pull-zone list --format table
hoppy pull-zone list --format text
```

## Global options

| Flag | Description |
|------|-------------|
| `--format json\|table\|text` | Output format (default: table) |
| `--debug` | Show HTTP request details |
| `--quiet` | Suppress non-essential output |
| `--yes` / `-y` | Skip confirmation prompts |

## Shell completions

```bash
# Bash
hoppy completions bash > ~/.local/share/bash-completion/completions/hoppy

# Zsh
hoppy completions zsh > ~/.zfunc/_hoppy

# Fish
hoppy completions fish > ~/.config/fish/completions/hoppy.fish
```

## Building from source

```bash
cargo build --release
```

## License

MIT
