# Copilot Instructions for Hoppy

## Repository Overview

Hoppy is an async Rust CLI application for managing bunny.net cloud services (CDN, storage, streaming, DNS, edge compute, security). It is designed for both human operators and AI agents. The CLI follows the pattern `hoppy <service> <action> [options]` inspired by az/gcloud/aws CLIs.

## Project Layout

```
hoppy/
├── crates/
│   ├── hoppy-cli/          # CLI binary (package: hoppy-cli, binary: hoppy)
│   │   ├── src/
│   │   │   ├── main.rs     # Entry point (#[tokio::main])
│   │   │   ├── cli.rs      # Clap derive structs (commands, flags, enums)
│   │   │   ├── auth.rs     # Env var reading (BUNNY_API_KEY, etc.)
│   │   │   ├── output.rs   # Output formatting (JSON, table, text)
│   │   │   └── commands/   # One handler per service
│   │   └── tests/e2e/      # End-to-end CLI tests (wiremock + insta)
│   ├── bunny-net-api/          # Hand-written bunny.net API clients (one crate, feature-gated modules)
│   │   └── src/
│   │       ├── lib.rs       # Feature-gated module declarations
│   │       ├── core/        # Pull zones, storage zones, DNS, video libraries
│   │       ├── shield/      # WAF, rate limiting, access lists, bot detection
│   │       ├── storage/     # Edge storage file operations (binary support)
│   │       ├── stream/      # Video library & video management (binary upload)
│   │       ├── compute/     # Edge scripting
│   │       ├── containers/  # Magic containers
│   │       ├── database/    # libSQL managed databases
│   │       └── recording/   # HTTP response recording helper (dev/test)
│   └── bunny-syslog-receiver/  # Standalone embedded syslog receiver
├── hoppy-knowledgebase/    # Design docs and research (Obsidian-compatible)
└── .github/workflows/      # CI: multi-platform build, clippy, rustfmt
```

## Rust Edition and Toolchain

- **Edition:** 2024 across all crates.
- **Async runtime:** Tokio (multi-threaded).
- **HTTP client:** reqwest 0.12.
- **Error handling:** `anyhow::Result<T>` with `.context()` for diagnostics. No typed errors.
- **Serialization:** serde + serde_json + serde_repr for integer enums.
- **CLI framework:** clap 4 with derive macros.
- **Output formatting:** tabled 0.17 for table output; JSON and tab-separated text also supported.
- **Test mocking:** wiremock 0.6.

## Build, Test, and Lint

```sh
cargo build                       # Build all crates
cargo test                        # Run all tests
cargo clippy -- -D warnings       # Lint (warnings are errors)
cargo fmt --check                 # Formatting check
```

CI runs these across Linux (gnu), macOS (x86_64 + aarch64), and Windows (msvc).

## Key Design Decisions

- **Hand-written API clients** (not code-generated). Progenitor codegen was evaluated and rejected because it cannot handle `application/octet-stream` binary bodies. Manual clients give tighter control over binary upload APIs and type refinements.
- **Single `bunny-net-api` crate** with services as feature-gated modules (`bunny_net_api::core`, `bunny_net_api::stream`, etc.). The former per-service `bunny-net-api-*` crates were merged in iter-32.
- **CLI package is `hoppy-cli`**, binary is `hoppy`. Install: `cargo install hoppy-cli`.
- **Three output formats** for every list operation: JSON, table, text (TSV).
- **Destructive operations** (delete) require user confirmation unless `--yes` is passed.

## Coding Conventions

### Serde / JSON Field Casing

- Most APIs (core, storage, stream): `#[serde(rename_all = "PascalCase")]`.
- Shield API: `#[serde(rename_all = "camelCase")]`.
- Acronym fields (MP4, URL, IP, DRM, SSL) need explicit `#[serde(rename = "...")]` because serde's PascalCase lowercases acronyms (e.g., `HasMP4Fallback` not `HasMp4Fallback`).

### Integer Enums

Use `serde_repr` with `#[repr(u8)]` and manual `Display` impls. Do not use string-based serde for numeric API enums.

### API Client Structure

Each client crate follows this module layout:
- `client.rs` — HTTP client struct with async methods returning `Result<T>`.
- `types.rs` — Serde structs and enums for request/response models.
- `lib.rs` — Public re-exports and module declarations.

All clients share a common pattern: construct with `Client::new(api_key)`, methods call reqwest, parse JSON response, extract structured `ApiError` on failure or fall back to status + body text.

### CLI Command Handlers

Each command handler defines compact display row structs:
```rust
#[derive(serde::Serialize, tabled::Tabled)]
struct PullZoneRow { /* fields with #[tabled(rename = "...")] */ }
impl From<&ApiType> for PullZoneRow { /* field mapping */ }
```

Output uses `print_data()` for paginated lists, `print_single()` for single items, `print_error()` for errors.

### Pagination

Always send `page` and `perPage` query parameters to the API (even with defaults) to receive the `PaginatedList<T>` envelope with `items`, `current_page`, `total_items`, `has_more_items`.

## Code Review Guidelines

When reviewing pull requests, pay close attention to:

1. **Serde correctness:** Verify field casing matches the target API (PascalCase vs camelCase). Check that acronym fields have explicit `#[serde(rename)]`. Ensure `#[serde(default)]` is used on optional/nullable fields.
2. **Error handling:** All fallible operations must use `Result` with `.context()`. No `.unwrap()` or `.expect()` in non-test code. Errors should propagate, not be silently swallowed.
3. **Async safety:** No blocking calls in async contexts. Verify `Send`/`Sync` across await points.
4. **Binary data handling:** Storage and stream crates handle raw bytes. Verify `Content-Type` headers and body encoding are correct for binary uploads/downloads.
5. **API key security:** Keys come from environment variables only. Never log, serialize, or expose API keys in error messages or debug output.
6. **Output format consistency:** Every new list command must support all three output formats (JSON, table, text). Display row structs must implement both `Serialize` and `Tabled`.
7. **Destructive operation safety:** Delete operations must prompt for confirmation unless `--yes` is set.
8. **Edition 2024 compliance:** Use current Rust 2024 idioms. No deprecated patterns.
9. **Clippy cleanliness:** Code must pass `cargo clippy -- -D warnings` with zero warnings.
10. **Test coverage:** New API client methods should have unit tests. Use wiremock for HTTP mocking.
11. **No unnecessary complexity:** Prefer simple, direct code. Don't add abstractions for single-use patterns. Three similar lines are better than a premature helper function.
12. **Dependency hygiene:** Do not add new dependencies without justification. Keep the dependency tree minimal.
