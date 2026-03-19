---
title: Release Engineering Research for hoppy v0.1.0
type: research
date: 2026-03-18
status: draft
tags:
  - release
  - ci-cd
  - packaging
  - github-actions
---

# Release Engineering Research for hoppy v0.1.0

## 1. GitHub Actions Release Workflow for Cross-Compiled Rust Binaries

### Target Matrix

| Target | OS Runner | Method |
|--------|-----------|--------|
| x86_64-unknown-linux-gnu | ubuntu-latest | cargo (native) |
| aarch64-unknown-linux-gnu | ubuntu-latest | cross-rs |
| x86_64-apple-darwin | macos-13 (Intel) or macos-latest + target | native cargo |
| aarch64-apple-darwin | macos-latest (M-series) | native cargo |
| x86_64-pc-windows-msvc | windows-latest | native cargo |

### Tool Comparison

**cross-rs** (recommended for Linux cross-compilation):
- Uses Docker containers with pre-configured toolchains
- Zero-setup cross-compilation
- Supports all needed targets
- Maintained under rust-cross org
- Gotcha: does not publish Linux ARM binary releases of cross itself (as of 2025-02)
- Best for: Linux aarch64 from x86_64 runner

**cargo-zigbuild**:
- Uses Zig as a linker; lighter than Docker
- Only supports Linux and macOS targets
- Reports of segfaults on some aarch64-linux builds
- Less battle-tested than cross-rs

**Native runners** (recommended where possible):
- macOS: GitHub now has M-series (aarch64) runners via `macos-latest`; Intel via `macos-13`
- Windows: `windows-latest` with MSVC toolchain works out of the box
- No Docker overhead, faster builds

### Recommended Approach

Follow ripgrep/bat pattern:
1. Trigger on tag push (e.g., `v*`)
2. Use matrix strategy with `os` + `target` + `use-cross` fields
3. Use native cargo for macOS (both arches), Windows, and Linux x86_64
4. Use cross-rs only for Linux aarch64
5. Upload release artifacts with `actions/upload-artifact` then create GitHub Release
6. Use `houseabsolute/actions-rust-cross` action which auto-selects cross vs cargo

### Ripgrep's Pattern

Ripgrep uses a nightly compiler for release builds (optimizations), defines TARGET_FLAGS and TARGET_DIR per matrix entry, and packages binaries into tarballs (.tar.gz for Unix, .zip for Windows) with completions and man pages included.

### References
- [ripgrep release.yml](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml)
- [actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)


## 2. Homebrew Tap

### What's Needed

1. Create repo `ractive/homebrew-tap` (name convention: `homebrew-<tapname>`)
2. Add a Ruby formula file in `Formula/hoppy.rb`
3. Users install via: `brew tap ractive/tap && brew install hoppy`

### Formula Structure

```ruby
class Hoppy < Formula
  desc "CLI for bunny.net services"
  homepage "https://github.com/ractive/hoppy"
  version "0.1.0"

  if OS.mac?
    if Hardware::CPU.arm?
      url "https://github.com/ractive/hoppy/releases/download/v0.1.0/hoppy-aarch64-apple-darwin.tar.gz"
      sha256 "CHECKSUM_HERE"
    else
      url "https://github.com/ractive/hoppy/releases/download/v0.1.0/hoppy-x86_64-apple-darwin.tar.gz"
      sha256 "CHECKSUM_HERE"
    end
  elsif OS.linux?
    if Hardware::CPU.arm?
      url "https://github.com/ractive/hoppy/releases/download/v0.1.0/hoppy-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "CHECKSUM_HERE"
    else
      url "https://github.com/ractive/hoppy/releases/download/v0.1.0/hoppy-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "CHECKSUM_HERE"
    end
  end

  def install
    bin.install "hoppy"
    # man1.install "hoppy.1"  # if man pages are bundled
    # bash_completion.install "completions/hoppy.bash"
    # zsh_completion.install "completions/_hoppy"
    # fish_completion.install "completions/hoppy.fish"
  end
end
```

### SHA256 Handling

- Compute at release time: `shasum -a 256 <file>` or `curl -L <url> | shasum -a 256`
- Automate: GitHub Actions step after uploading assets computes checksums and updates formula
- Tool: `formulaic` crate can auto-generate formulas from GitHub release metadata

### Automation

Use a GitHub Actions step in the release workflow that:
1. Computes SHA256 for each artifact
2. Updates the formula in the homebrew-tap repo (via PAT or GitHub App)
3. Commits and pushes

### References
- [formulaic](https://github.com/ceejbot/formulaic)
- [Automating Homebrew Tap Updates](https://builtfast.dev/blog/automating-homebrew-tap-updates-with-github-actions/)
- [Guide to creating a Homebrew tap](https://kristoffer.dev/blog/guide-to-creating-your-first-homebrew-tap/)


## 3. Windows / winget

### Simplest Path: Ship a .zip with Portable Installer

winget supports ZIP archives since v1.5. For a CLI tool, the simplest approach:

1. Ship `hoppy-x86_64-pc-windows-msvc.zip` containing `hoppy.exe`
2. Create a winget manifest with `InstallerType: zip` and `NestedInstallerType: portable`
3. Submit manifest to `microsoft/winget-pkgs` repo via PR

### Manifest Structure (multi-file format)

Needed files in `manifests/r/ractive/hoppy/0.1.0/`:
- `ractive.hoppy.yaml` (version manifest)
- `ractive.hoppy.installer.yaml` (installer manifest)
- `ractive.hoppy.locale.en-US.yaml` (default locale)

Key fields in installer manifest:
```yaml
PackageIdentifier: ractive.hoppy
PackageVersion: 0.1.0
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: hoppy.exe
    PortableCommandAlias: hoppy
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/ractive/hoppy/releases/download/v0.1.0/hoppy-x86_64-pc-windows-msvc.zip
    InstallerSha256: CHECKSUM_HERE
```

### Alternatives

- **MSI via WiX**: More work, but gives proper install/uninstall in Add/Remove Programs. WiX 4+ supports Rust projects but requires learning WiX toolset.
- **NSIS**: Overkill for a CLI tool.
- **Recommendation**: Start with zip+portable for v0.1.0; consider MSI later if users request it.

### Tooling

Use `wingetcreate` CLI to generate and validate manifests: `wingetcreate new <url>`

### References
- [winget-pkgs installer manifest schema](https://github.com/microsoft/winget-pkgs/blob/master/doc/manifest/schema/1.6.0/installer.md)
- [winget-pkgs repo](https://github.com/microsoft/winget-pkgs)


## 4. Linux Packages (deb/rpm)

### cargo-deb

Generates .deb packages directly from Cargo.toml metadata.

**Cargo.toml config needed:**
```toml
[package.metadata.deb]
maintainer = "Your Name <email>"
copyright = "2026, Your Name"
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
    ["target/release/hoppy", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/hoppy/", "644"],
    # ["target/release/completions/hoppy.bash", "usr/share/bash-completion/completions/hoppy", "644"],
    # ["target/release/completions/_hoppy", "usr/share/zsh/vendor-completions/", "644"],
    # ["target/release/completions/hoppy.fish", "usr/share/fish/vendor_completions.d/", "644"],
    # ["target/release/man/hoppy.1", "usr/share/man/man1/", "644"],
]
```

Build: `cargo deb` (auto-builds release binary + packages)

### cargo-generate-rpm

Generates .rpm packages. Replaces the deprecated cargo-rpm.

**Cargo.toml config needed:**
```toml
[package.metadata.generate-rpm]
assets = [
    { source = "target/release/hoppy", dest = "/usr/bin/hoppy", mode = "0755" },
    { source = "README.md", dest = "/usr/share/doc/hoppy/README.md", mode = "0644" },
]
```

Build: `cargo build --release && cargo generate-rpm`

### CI Integration

- Build the binary first (potentially with cross for aarch64)
- Run cargo-deb/cargo-generate-rpm as a post-build step
- Upload .deb and .rpm as release artifacts
- Consider: for aarch64, you need the cross-compiled binary in the target dir before packaging

### Priority for v0.1.0

Start with .deb only (larger user base). Add .rpm in a follow-up.

### References
- [cargo-deb](https://github.com/kornelski/cargo-deb)
- [cargo-generate-rpm](https://crates.io/crates/cargo-generate-rpm)
- [Guide to deb and rpm for Rust](https://dev.to/mbayoun95/comprehensive-guide-to-generating-deb-and-rpm-packages-for-rust-applications-41h7)


## 5. Shell Completion Installation

### How Popular Tools Handle It

**bat**: `bat --completion bash/zsh/fish` prints completions to stdout. User redirects to appropriate path. Recent addition.

**ripgrep**: Ships pre-generated completion files in release tarballs. Also available via `rg --generate complete-bash` etc.

**starship**: `starship completions bash/zsh/fish` prints to stdout.

**clap pattern**: Use `clap_complete` to generate at runtime. hoppy already depends on `clap_complete = "4"`.

### Recommended Approach for hoppy

Provide a subcommand like `hoppy completions <shell>` that prints to stdout:

```
hoppy completions bash > ~/.local/share/bash-completion/completions/hoppy
hoppy completions zsh > ~/.zfunc/_hoppy
hoppy completions fish > ~/.config/fish/completions/hoppy.fish
```

### Standard Installation Paths

| Shell | User path | System path (for packages) |
|-------|-----------|---------------------------|
| Bash | `~/.local/share/bash-completion/completions/` | `/usr/share/bash-completion/completions/` |
| Zsh | `~/.zfunc/` (needs fpath addition) | `/usr/share/zsh/vendor-completions/` |
| Fish | `~/.config/fish/completions/` | `/usr/share/fish/vendor_completions.d/` |

### Package Bundling vs On-Demand

- **Packages (deb/rpm/brew)**: Bundle pre-generated completions in system paths
- **cargo install / manual**: User runs `hoppy completions <shell>` to generate on demand
- **Release tarballs**: Include completions/ directory with pre-generated files

### References
- [CLI Shell Completions in Rust](https://kbknapp.dev/shell-completions/)
- [bat --completion](https://github.com/sharkdp/bat)


## 6. Man Page Generation

### clap_mangen

- Part of the clap ecosystem, 268K monthly downloads
- Generates roff-format man pages from clap `Command` definitions
- Single source of truth: CLI definition drives both --help and man page

### Integration Options

1. **build.rs**: Auto-generates on every build. Can slow iteration. Not recommended.
2. **cargo xtask**: Run `cargo xtask man` on demand or in CI. Recommended.
3. **Binary subcommand**: `hoppy --generate man` generates to stdout at runtime.

### Effort vs Value

- **Low effort**: ~20 lines of code in an xtask or build script
- **Medium value**: Nice for packages (deb/rpm/brew install man pages), but most users use `--help`
- **Recommendation**: Add for v0.1.0 if packaging; skip if only doing GitHub releases. Man pages in tarballs are a nice professional touch (ripgrep does this).

### References
- [clap_mangen](https://crates.io/crates/clap_mangen)
- [Rust CLI book: rendering docs](https://rust-cli.github.io/book/in-depth/docs.html)


## 7. cargo install Support

### Current State

hoppy already has the basics: `name`, `version`, `edition`, `description`, `license` in Cargo.toml.

### Additional Metadata Needed for crates.io

```toml
[package]
repository = "https://github.com/ractive/hoppy"
homepage = "https://github.com/ractive/hoppy"
keywords = ["bunny", "cdn", "cli"]
categories = ["command-line-utilities"]
readme = "README.md"
```

### Workspace Gotchas

1. **Path dependencies must be published first**: All `bunny-api-*` crates must be on crates.io before `hoppy` can be published. They must use version numbers, not just `path = ...`.
2. **Publishing order matters**: Publish leaf crates first (bunny-api-core), then dependents, then hoppy last.
3. **Lock file ignored**: `cargo install` ignores Cargo.lock by default. Pin important deps or use `--locked` flag. Document `cargo install hoppy --locked` for reproducible builds.
4. **Workspace publish order**: Cargo 1.90 (Sept 2025 stable) adds workspace-level publishing support, simplifying multi-crate publishes.
5. **Cargo.toml for sub-crates**: Each `bunny-api-*` crate needs full metadata (version, license, description, repository) to publish to crates.io.

### Dual Dependencies

For workspace crates, use both path and version:
```toml
bunny-api-core = { version = "0.1.0", path = "crates/bunny-api-core" }
```

### Alternative: Don't Publish Sub-Crates

If the API crates aren't meant for public consumption, you can:
- Only publish the `hoppy` binary crate to crates.io
- But this requires vendoring or the sub-crates being available
- Simpler: publish all crates, mark API crates with a note they're internal


## 8. Other v0.1.0 Release Requirements

### LICENSE File
- Already declared `license = "MIT"` in Cargo.toml
- Need actual `LICENSE` or `LICENSE-MIT` file in repo root
- Consider dual license MIT OR Apache-2.0 (Rust ecosystem convention)

### CHANGELOG
- Create `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format
- Or use [git-cliff](https://github.com/orhun/git-cliff) to auto-generate from conventional commits

### README.md
- Installation instructions (brew, cargo install, binary download)
- Quick-start usage examples
- Feature overview
- Badge for CI status

### .github Files
- `CONTRIBUTING.md` (optional for v0.1.0)
- Issue/PR templates (optional)

### Pre-publish Checklist

1. `cargo fmt --check` passes
2. `cargo clippy` clean
3. `cargo test` passes
4. `cargo publish --dry-run` succeeds (for each crate)
5. No secrets in source
6. `.env` in `.gitignore`
7. LICENSE file present
8. README exists with install instructions
9. All crate metadata complete (description, license, repository)
10. Git tag created: `git tag -a v0.1.0 -m "v0.1.0"`

### Suggested Release Day Workflow

1. Update versions in all Cargo.toml files
2. Update CHANGELOG.md
3. Commit: "Release v0.1.0"
4. Tag: `git tag -a v0.1.0 -m "v0.1.0"`
5. Push: `git push origin main --tags`
6. GitHub Actions builds binaries, creates Release, uploads artifacts
7. Publish to crates.io (manual or CI): `cargo publish -p bunny-api-core`, etc.
8. Update Homebrew tap formula
9. Submit winget manifest PR (optional, can defer)

### References
- [sharkdp's release checklist](https://dev.to/sharkdp/my-release-checklist-for-rust-programs-1m33)
- [Orhun's automated Rust releases](https://blog.orhun.dev/automated-rust-releases/)
- [Crate release checklist gist](https://gist.github.com/BartMassey/a8bf0d5fee366f55b6ed90c3c55ef20d)
