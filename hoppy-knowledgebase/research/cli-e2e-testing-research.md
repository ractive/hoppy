---
title: CLI End-to-End Testing Research
type: research
date: 2026-03-19
status: complete
tags:
  - testing
  - e2e
  - cli
  - rust
  - mock-server
  - record-replay
---

# CLI End-to-End Testing: Comprehensive Research

## 1. Rust CLI Testing Ecosystem

### 1.1 assert_cmd + predicates (Primary Recommendation)

The `assert_cmd` crate is the de facto standard for testing Rust CLI binaries end-to-end. It wraps `std::process::Command` with ergonomic assertions.

**Core pattern:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_version() {
    Command::cargo_bin("my_app")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("1.0.0"));
}

#[test]
fn test_error_case() {
    Command::cargo_bin("my_app")
        .unwrap()
        .arg("--nonexistent")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("error:"));
}
```

**Key features:**
- `Command::cargo_bin("name")` - finds and runs the compiled binary
- `.assert().success()` / `.failure()` - check exit code
- `.stdout()` / `.stderr()` - match output with predicates
- `predicates::str::is_match(regex)` - regex matching
- `predicates::str::contains(text)` - substring matching
- Multiple args via `.args(&["--flag", "value"])`

**Best practice:** Keep tests readable; a bit of repetition is fine. Helper functions should group by testing purpose, not just reduce boilerplate.

### 1.2 assert_fs (File System Testing)

The `assert_fs` crate creates temporary directories and files for tests, with automatic cleanup when variables go out of scope.

```rust
use assert_fs::prelude::*;
use assert_fs::TempDir;

#[test]
fn test_file_processing() {
    let temp = TempDir::new().unwrap();
    let input = temp.child("input.txt");
    input.write_str("hello world").unwrap();

    Command::cargo_bin("my_app")
        .unwrap()
        .arg(input.path())
        .assert()
        .success();

    temp.child("output.txt")
        .assert(predicates::path::exists());
}
```

### 1.3 trycmd (Snapshot Testing for CLI)

`trycmd` enables bulk snapshot testing of CLI commands using either TOML or Markdown files. Inspired by `trybuild` and `cram`.

**TOML format** (`tests/cmd/version.toml`):
```toml
bin.name = "my_app"
args = ["--version"]
```
With companion `tests/cmd/version.stdout`:
```
my_app 1.0.0
```

**Markdown format** (`tests/cmd/basics.trycmd`):
```
$ my_app --version
my_app 1.0.0

$ my_app --help
? 0
```

**Key features:**
- `...` matches multiple lines (elision)
- `[..]` matches any characters on a line
- `[EXE]` handles platform-specific executable extensions
- `[ROOT]`/`[CWD]` normalize paths
- `TRYCMD=overwrite cargo test` updates snapshots
- `TRYCMD=dump` generates initial snapshots
- `fs.sandbox = true` in TOML enables file system verification
- `*.in/` and `*.out/` directories for file system state

**When to use:** Ideal for testing many commands with simple input/output. Use `assert_cmd` for more complex assertions.

### 1.4 rexpect (Interactive CLI Testing)

For testing interactive prompts, `rexpect` (similar to Python's `pexpect`) allows spawning a CLI process and interacting with its stdin/stdout.

```rust
use rexpect::spawn;

#[test]
fn test_interactive_prompt() {
    let mut p = spawn("my_app init", Some(5000)).unwrap();
    p.exp_string("Enter project name:").unwrap();
    p.send_line("my-project").unwrap();
    p.exp_string("Created my-project").unwrap();
}
```

Also see `expectrl` for a more maintained alternative.

### 1.5 lit (LLVM-style Testing)

The `lit` crate provides LLVM FileCheck-style testing without requiring Python or LLVM. Tests embed `CHECK:` assertions in comments for sparse output matching. Niche but powerful for compiler-like tools.

### 1.6 snapbox

Lower-level snapshot testing library used by `trycmd` internally. Useful for one-off cases where you need trycmd-like behavior with more customization.

---

## 2. How Major Rust CLIs Test

### 2.1 Cargo (Rust's Build System)

Cargo has one of the most sophisticated CLI test setups:

- **Location:** `tests/testsuite/<command>.rs` (functional) and `tests/testsuite/<command>/<case>/mod.rs` (UI/snapshot)
- **`#[cargo_test]` attribute:** Sets up sandbox isolation under `target/tmp/cit/`
- **`project()` helper:** Programmatically creates test projects with `Cargo.toml` and source files
- **`p.cargo("build")` / `p.cargo("run")`:** Executes cargo commands and captures output
- **Pattern matching:** `str![[...]]` macros with `snapbox` for flexible output assertions
- **`SNAPSHOTS=overwrite`:** Updates snapshot expectations
- **Sandbox isolation:** Each test gets its own filesystem with a fake `$HOME`
- **No network:** Uses `support::registry::Package` and `support::git` for offline dependency simulation

### 2.2 ripgrep

- Integration tests in `tests/tests.rs` (single entry point)
- Uses custom test harness with `std::process::Command`
- Tests exercise the binary directly, checking stdout/stderr/exit codes
- Extensive coverage of regex, file type, and encoding edge cases

### 2.3 General Pattern Across Rust CLIs

Most popular Rust CLIs (fd, bat, etc.) use some combination of:
1. `assert_cmd` for binary execution
2. `predicates` for output matching
3. `assert_fs` or `tempfile` for filesystem isolation
4. Custom helpers for domain-specific assertions

---

## 3. Non-Rust CLI Testing Frameworks

### 3.1 Shell/Bash Frameworks

| Framework | Language | Style | Parallel | Key Strength |
|-----------|----------|-------|----------|-------------|
| **bats-core** | Bash | Custom syntax | Yes | Most popular; TAP output; CI-friendly |
| **ShellSpec** | Shell | BDD DSL | Yes | Rich assertions; parameterized tests |
| **bashunit** | Bash | Modern | Yes | Newest (2023); fast; good docs |
| **shUnit2** | Bash | xUnit | No | Oldest; pure Bash; stable |
| **cram** | Python | Snapshot | No | Language-agnostic CLI snapshot testing |
| **shelltestrunner** | Haskell | Declarative | No | Minimal; specify command/stdin/expected output/exit code |

**Recommendation for language-agnostic CLI testing:** `bats-core` for imperative tests, `cram` for snapshot-style tests.

### 3.2 Python

- **click.testing.CliRunner:** In-process testing for Click CLIs without subprocess overhead. Captures stdout, stderr, exit code. Supports input simulation for prompts.
- **subprocess + pytest:** Direct binary testing. Use `subprocess.run()` or `subprocess.Popen` for interactive tests. Combine with `pytest.fixture` and `tmp_path`.
- **cram:** Write tests in a shell-script-like format with expected output. Language-agnostic.

### 3.3 Node.js / TypeScript

- **@oclif/test:** Official testing utilities for oclif CLIs. Provides `captureOutput`, `runCommand`, `runHook`. Works with mocha/jest/vitest.
- **bats-core:** Often used for Node CLI E2E testing too, since it tests the actual binary.
- **execa:** Node.js process execution library frequently used in test suites.

### 3.4 Go (Cobra-based CLIs)

- **Capture stdout/stderr on cobra.Command:** Execute commands programmatically, capture `bytes.Buffer` output.
- **Dependency injection:** Use factory patterns so commands accept interfaces, inject mocks for testing.
- **httptest.NewServer:** Go stdlib HTTP test server for API mocking.
- **gock:** HTTP traffic mocking library often paired with Cobra CLIs.
- **Table-driven tests:** Standard Go pattern; define input/expected-output pairs in a slice.

---

## 4. Mock Server Approaches for API-Backed CLIs

### 4.1 How Major CLIs Handle API Mocking

| CLI Tool | Language | Approach |
|----------|----------|----------|
| **gh (GitHub CLI)** | Go | Custom `pkg/httpmock` package; table-driven tests; no real API calls. Stubs/mocks for HTTP, git, filesystem. |
| **stripe-cli** | Go | Dedicated `stripe-mock` server (Go binary) that responds like real Stripe API. Fixture-based responses from OpenAPI spec. Stateless. |
| **aws-cli** | Python | `moto` library simulates AWS services in-memory. Supports decorator and server modes. |
| **gcloud** | Python | Internal mock infrastructure; not publicly documented. |
| **heroku CLI** | Node/TS | `@oclif/test` + `nock` for HTTP mocking. |

### 4.2 Mock Server Tools

| Tool | Language | Record/Replay | Proxy Mode | Standalone Server |
|------|----------|--------------|------------|-------------------|
| **httpmock** (Rust) | Rust | Yes (YAML files) | Yes | Yes |
| **wiremock-rs** | Rust | No | No | No (in-process only) |
| **WireMock** (Java) | Java | Yes | Yes | Yes (standalone JAR) |
| **WireMock Cloud** | SaaS | Yes | Yes | Yes |
| **mockoon** | Node | No (manual) | No | Yes (GUI + CLI) |
| **Prism** | Node | No | Yes (validation) | Yes |
| **mitmproxy** | Python | Yes | Yes | Yes |
| **nock** | Node | Yes | No | No (in-process) |
| **moto** | Python | N/A (simulates) | Server mode | Yes |
| **gock** | Go | No | No | No (in-process) |

### 4.3 httpmock (Rust) -- Record & Replay Detail

`httpmock` is the only Rust mock crate with built-in record/replay. Two recording strategies:

**Forwarding mode** (easier; requires changing client base URL):
```rust
let server = MockServer::start();
server.forward_to("https://api.real-service.com", |rule| {
    rule.filter(|when| { when.path_prefix("/v1"); });
});
let recording = server.record(|when| { when.path_prefix("/v1"); });
// ... run tests that hit the mock server ...
recording.save("my_scenario");
```

**Proxy mode** (no URL change needed; client must support proxy config):
```rust
let server = MockServer::start();
server.proxy(|rule| {
    rule.filter(|when| { when.any_request(); });
});
let recording = server.record(|when| { when.any_request(); });
// ... run tests with HTTP_PROXY set to mock server ...
recording.save("my_scenario");
```

**Playback:**
```rust
let server = MockServer::start();
server.playback("target/httpmock/recordings/my_scenario_<timestamp>.yaml");
// Requests matching recorded patterns get recorded responses
```

Recordings are saved as YAML in `target/httpmock/recordings/`. Timestamped filenames.

### 4.4 wiremock-rs vs httpmock

| Feature | wiremock-rs | httpmock |
|---------|-------------|----------|
| Async-only | Yes | No (sync + async) |
| Record/Replay | No | Yes |
| Proxy mode | No | Yes |
| Standalone server | No | Yes |
| Request matching | Extensible traits | Built-in helpers |
| YAML mock definitions | No | Yes |
| Parallel test isolation | Yes (random ports) | Yes (random ports) |
| Expectation verification | Yes (spy/verify counts) | Yes |

**Verdict:** For an API-backed CLI that needs record/replay, `httpmock` is clearly the better choice.

---

## 5. Record/Replay Patterns

### 5.1 The VCR Pattern

Originated in Ruby (`vcr` gem). The concept: record real HTTP interactions once, save as "cassettes," replay in future test runs.

**Implementations across languages:**
| Language | Library | Format |
|----------|---------|--------|
| Ruby | vcr | YAML cassettes |
| Python | vcrpy / pytest-recording | YAML/JSON cassettes |
| Node | nock (record mode) | JSON |
| Go | go-vcr | YAML cassettes |
| Rust | httpmock (record mode) | YAML |
| .NET | Scotch / Betamax.Net | - |
| PHP | php-vcr | - |
| Java | WireMock | JSON mappings |

### 5.2 Best Practices for Record/Replay

1. **Redact secrets:** Strip API keys, tokens, and auth headers from recordings before committing.
2. **Deterministic ordering:** Name cassettes explicitly; don't rely on test execution order.
3. **Refresh periodically:** Re-record cassettes when the real API changes.
4. **CI mode:** Run in replay-only mode in CI; record only locally.
5. **Diff-friendly format:** YAML or formatted JSON so changes are visible in code review.

### 5.3 httpmock Record/Replay for Rust (Recommended Approach)

The workflow for a Rust CLI:
1. **Record:** Run tests with `RECORD=1` env var, pointing httpmock to real API via forwarding/proxy.
2. **Save:** Recordings go to `target/httpmock/recordings/` as YAML.
3. **Commit:** Copy YAML fixtures to `tests/fixtures/` (after redacting secrets).
4. **Replay:** Tests load fixtures in normal mode; no network required.
5. **CI:** Always replay from committed fixtures.

---

## 6. Dual-Mode Testing (Real API vs Mock)

### 6.1 Architecture Pattern

```
                    ┌─────────────────┐
                    │   Test Suite     │
                    │  (assert_cmd)   │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
          ┌────────┤  Mode Switch     ├────────┐
          │        │  (env var)       │        │
          │        └─────────────────-┘        │
          │                                    │
  ┌───────▼───────┐                 ┌──────────▼──────────┐
  │  httpmock      │                │  Real API            │
  │  (playback)    │                │  (test/sandbox env)  │
  └───────────────-┘                └─────────────────────-┘
```

### 6.2 Implementation Strategy

1. **Environment variable toggle:** e.g., `HOPPY_TEST_MODE=mock|live`
2. **Base URL override:** CLI accepts `--api-url` or `BUNNY_API_URL` env var. Point to mock server in mock mode, real API in live mode.
3. **Shared test functions:** Write test logic once; the setup/teardown differs based on mode.
4. **httpmock forwarding mode as bridge:** In record mode, httpmock sits between CLI and real API, capturing interactions. Same test code runs in both modes.

### 6.3 Concrete Rust Pattern

```rust
fn setup_server() -> (MockServer, String) {
    let server = MockServer::start();
    if std::env::var("RECORD").is_ok() {
        // Forward to real API and record
        server.forward_to("https://api.bunny.net", |rule| {
            rule.filter(|when| { when.any_request(); });
        });
        let _recording = server.record(|when| { when.any_request(); });
    } else {
        // Playback from fixtures
        server.playback("tests/fixtures/scenario.yaml");
    }
    let url = server.base_url();
    (server, url)
}

#[test]
fn test_list_zones() {
    let (server, base_url) = setup_server();
    Command::cargo_bin("hoppy")
        .unwrap()
        .env("BUNNY_API_URL", &base_url)
        .args(&["dns", "zone", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("example.com"));
}
```

---

## 7. Test Report Generation

### 7.1 markdown-test-report

A Rust tool that converts `cargo test` JSON output to Markdown.

**Usage:**
```bash
cargo test -- -Z unstable-options --report-time --format json > test-results.json
markdown-test-report --input test-results.json --output test-report.md
```

**Features:**
- Test summaries with pass/fail counts
- Execution timing
- Git metadata integration
- Configurable verbosity (summary-only mode available)
- No built-in checkbox support, but output is Markdown so `- [x]` / `- [ ]` could be templated

### 7.2 Custom Report Generation

For checkbox-style reports, a simple script could parse `cargo test --format json` output and generate:
```markdown
## Test Results
- [x] test_list_zones (0.12s)
- [x] test_create_zone (0.45s)
- [ ] test_delete_zone -- FAILED (0.03s)
```

### 7.3 cargo-nextest

Alternative test runner that produces JUnit XML, which can be transformed to Markdown. Faster parallel execution than `cargo test`.

---

## 8. Recommendations for This Project (hoppy CLI)

### Primary Stack

| Layer | Tool | Purpose |
|-------|------|---------|
| Binary execution | `assert_cmd` | Run `hoppy` binary, assert exit code + output |
| Output matching | `predicates` | Flexible stdout/stderr assertions |
| File system | `assert_fs` | Temporary dirs for config files |
| Snapshot tests | `trycmd` | Bulk test simple command outputs |
| HTTP mocking | `httpmock` | Record/replay API interactions |
| Interactive tests | `rexpect` | If/when interactive prompts exist |
| Test reports | `markdown-test-report` or custom | CI report generation |

### Suggested Test Structure

```
tests/
├── cli_tests.rs          # trycmd entry point
├── cmd/                   # trycmd snapshot files
│   ├── help.trycmd
│   ├── version.toml
│   └── dns_zone_list.toml
├── e2e/                   # assert_cmd integration tests
│   ├── mod.rs
│   ├── dns.rs
│   ├── stream.rs
│   └── storage.rs
├── fixtures/              # httpmock recorded YAML files
│   ├── dns_zone_list.yaml
│   ├── stream_library_list.yaml
│   └── pull_zone_list.yaml
└── support/               # shared test helpers
    ├── mod.rs
    └── mock_server.rs     # httpmock setup/teardown
```

### Dev Dependencies

```toml
[dev-dependencies]
assert_cmd = "2"
assert_fs = "1"
predicates = "3"
trycmd = "0.15"
httpmock = "0.8"
```

### Dual-Mode Test Workflow

1. **Default (CI):** `cargo test` -- runs with httpmock playback from committed YAML fixtures
2. **Record mode:** `RECORD=1 BUNNY_API_KEY=xxx cargo test` -- hits real Bunny API, records responses
3. **Live mode:** `LIVE=1 BUNNY_API_KEY=xxx cargo test` -- hits real API directly, no recording

---

## Sources

- [Rust CLI Book: Testing](https://rust-cli.github.io/book/tutorial/testing.html)
- [Testing Rust CLI apps with assert_cmd (alexwlchan, 2025)](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert-cmd/)
- [Approaches for E2E Testing in Rust CLI Applications (Sling Academy)](https://www.slingacademy.com/article/approaches-for-end-to-end-testing-in-rust-cli-applications/)
- [trycmd docs](https://docs.rs/trycmd)
- [Cargo Contributor Guide: Writing Tests](https://doc.crates.io/contrib/tests/writing.html)
- [Testing Rust CLIs with LLVM lit (Neil Henning)](https://www.neilhenning.dev/posts/rust-lit/)
- [httpmock Recording](https://httpmock.rs/record-and-playback/recording/)
- [httpmock Playback](https://httpmock.rs/record-and-playback/playback/)
- [wiremock-rs GitHub](https://github.com/LukeMathWalker/wiremock-rs)
- [httpmock GitHub](https://github.com/httpmock/httpmock)
- [WireMock Record and Playback](https://wiremock.org/docs/record-playback/)
- [7 API Mocking Patterns for 2025](https://dev.to/eggqing/7-api-mocking-patterns-every-2025-dev-pipeline-needs-3boj)
- [Bash Test Framework Comparison (dodie)](https://github.com/dodie/testing-in-bash)
- [bats-core](https://github.com/bats-core/bats-core)
- [shelltestrunner](https://github.com/simonmichael/shelltestrunner)
- [oclif Testing](https://oclif.io/docs/testing/)
- [Testing Cobra CLI in Go](https://gianarb.it/blog/golang-mockmania-cli-command-with-cobra)
- [Cobra CLI Testing with DI](https://jonesrussell.github.io/blog/golang/testing/2024/07/24/a-nod-to-golang-testing-cobra-cli-applications-with-dependency-injection.html)
- [GitHub CLI Architecture](https://www.augmentcode.com/open-source/cli/cli)
- [stripe-mock](https://github.com/stripe/stripe-mock)
- [moto (AWS mock)](https://github.com/getmoto/moto)
- [VCR (Ruby)](https://github.com/vcr/vcr)
- [markdown-test-report](https://github.com/ctron/markdown-test-report)
- [rexpect](https://github.com/rust-cli/rexpect)
- [Click Testing Docs](https://click.palletsprojects.com/en/stable/testing/)
- [Python CLI Testing (Real Python)](https://realpython.com/python-cli-testing/)

## Related
- [[iterations/e2e-test-harness-plan]] — plan derived from this research
- [[iterations/rust-e2e-rewrite-plan]] — rewrite plan
- [[testing/test-plan-v0.1.0]] — test plan
