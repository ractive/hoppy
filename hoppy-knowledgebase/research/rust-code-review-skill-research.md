---
date: 2026-03-18
status: completed
tags:
- rust
- code-review
- best-practices
- edition-2024
- skill-building
title: Rust Code Review Skill Research
type: research
---

# Rust Code Review Skill Research

Comprehensive research for building a Rust code review skill. Covers edition 2024 changes, review checklists, anti-patterns, and PR-specific concerns.

---

## 1. Rust 2024 Edition Changes

Released with Rust 1.85.0 on February 20, 2025. The largest edition to date, with many small but significant quality-of-life improvements.

### 1.1 Language Changes

#### Unsafe Tightening

- **`unsafe_op_in_unsafe_fn` warns by default**: Unsafe operations inside `unsafe fn` now require an explicit `unsafe {}` block. Previously, the entire body of an unsafe function was implicitly unsafe.

  ```rust
  // BAD (edition 2024 warns):
  unsafe fn do_thing(ptr: *const i32) -> i32 {
      *ptr  // implicit unsafe, no block
  }

  // GOOD:
  unsafe fn do_thing(ptr: *const i32) -> i32 {
      unsafe { *ptr }  // explicit unsafe block with safety reasoning
  }
  ```

- **`unsafe extern` blocks**: Extern blocks now require the `unsafe` keyword: `unsafe extern "C" { ... }`.

- **Unsafe attributes**: `#[export_name]`, `#[link_section]`, and `#[no_mangle]` must now be marked `unsafe`:

  ```rust
  // BAD (edition 2024 error):
  #[no_mangle]
  pub extern "C" fn my_func() {}

  // GOOD:
  #[unsafe(no_mangle)]
  pub extern "C" fn my_func() {}
  ```

- **`static mut` references denied by default**: The `static_mut_refs` lint is now deny-by-default. Taking a reference to a `static mut` is an error. Use `std::sync::Mutex`, `std::sync::OnceLock`, atomics, or raw pointers instead.

- **Newly unsafe functions**: `std::env::set_var`, `std::env::remove_var`, and `std::os::unix::process::CommandExt::before_exec` are now unsafe.

#### RPIT Lifetime Capture Rules

In edition 2024, return-position `impl Trait` (RPIT) opaque types automatically capture ALL in-scope type and lifetime parameters. In 2021, only type parameters were captured.

```rust
// In 2021, this didn't capture 'a:
fn foo<'a>(x: &'a str) -> impl Display { ... }

// In 2024, 'a is automatically captured.
// To opt out, use explicit `use<>` bounds:
fn foo<'a>(x: &'a str) -> impl Display + use<> { ... }
```

This can cause compilation failures when migrating if the hidden type doesn't actually live long enough for the captured lifetime. `cargo fix` handles most cases by inserting `+ use<..>` bounds.

#### Let Chains in `if` and `while`

`let` expressions can now be chained with `&&` inside `if` and `while` conditions:

```rust
// NEW in 2024:
if let Some(x) = opt_a && let Some(y) = opt_b && x > 0 {
    use(x, y);
}
```

#### Match Ergonomics Restrictions

Some pattern combinations that were previously allowed are now errors to avoid confusion and enable future improvements. Specifically, capture modifiers (`ref`, `mut`, `ref mut`) are disallowed in patterns using match ergonomics (auto-dereferencing).

#### Never Type (`!`) Coercion Changes

Changes to how the never type `!` coerces, affecting fallback behavior in some edge cases.

#### Macro Changes

- **`expr` fragment specifier** now also matches `const` and `_` expressions. Use `expr_2021` to preserve old behavior.
- **`missing_fragment_specifier`** is now a hard error (was a warning).

#### Reserved Syntax

- `gen` is now a reserved keyword (for future generator blocks). Use `r#gen` if needed as an identifier.
- `#"foo"#` style guarded string literals and `##` tokens are reserved for future use.

#### Prelude Additions

`Future` and `IntoFuture` are added to the 2024 edition prelude. This can cause ambiguity if you have your own traits with conflicting method names.

### 1.2 Temporary Scope Changes

- **If-let temporaries**: Changed scope of temporaries in `if let` expressions.
- **Tail expression temporaries**: Temporaries from the tail expression of a block are now dropped BEFORE local variables, fixing a class of bugs where temporaries outlived expectations.

### 1.3 Tooling Changes

- **Rustfmt**: Can now have its own edition, defaulting to the crate edition. May reformat some code differently.
- **Rustdoc**: Doc tests are now combined into a single binary where possible, improving compile times for `cargo test --doc`.

### 1.4 Migration

`cargo fix --edition` handles most migrations automatically. Key manual interventions:
- Adding `unsafe` to extern blocks and attributes
- Handling RPIT capture rule changes
- Renaming identifiers that clash with `gen` keyword
- Reviewing match ergonomics changes

---

## 2. Rust Code Review Checklist

### 2.1 Ownership, Borrowing, and Lifetimes

**Check for:**
- Ownership is clear: for any value, you can identify what owns it and when it is dropped
- Only one mutable reference exists at any point (compiler enforces, but complex patterns may hide issues)
- No unnecessary `clone()` calls; prefer borrowing
- Lifetime annotations are minimal and correct
- No lifetime proliferation that makes APIs hard to use

**Bad patterns:**

```rust
// Unnecessary clone to satisfy borrow checker
let name = self.name.clone();
do_something(&name);

// Better: borrow directly
do_something(&self.name);
```

### 2.2 Error Handling

**Libraries should use `thiserror`:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid response: {message}")]
    InvalidResponse { message: String },

    #[error("not found: {resource}")]
    NotFound { resource: String },
}
```

**Applications should use `anyhow`:**

```rust
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    toml::from_str(&content)
        .context("failed to parse config")
}
```

**Check for:**
- No `.unwrap()` or `.expect()` in library code or production paths (acceptable in tests and proven-safe cases with a comment)
- Error types preserve the causal chain (`#[source]` or `#[from]`)
- Error messages are lowercase, no trailing punctuation (Rust convention)
- Context is added at call sites, not just propagated with bare `?`
- Error types derive `Debug`
- No `Box<dyn Error>` in public library APIs (use typed errors)
- Panics are documented with `# Panics` doc section

### 2.3 Unsafe Code

**Every `unsafe` block MUST have a `// SAFETY:` comment explaining:**
1. What invariants are being relied upon
2. Why those invariants hold at this call site
3. Under what conditions the safety contract could break

**Check for:**
- Minimal unsafe scope; package unsafety into safe abstractions
- FFI calls wrapped in safe functions
- No undefined behavior: null pointer dereference, data races, invalid references, aliasing violations
- Cross-module or cross-crate safety invariants are documented at BOTH sites
- If safe code must uphold invariants for unsafe code elsewhere, that safe code is annotated

```rust
// GOOD:
// SAFETY: `ptr` was allocated by `alloc_widget` and has not been freed.
// The caller guarantees exclusive access via the `&mut self` receiver.
unsafe { ptr::drop_in_place(self.ptr) }

// BAD:
unsafe { ptr::drop_in_place(self.ptr) }  // no safety comment
```

### 2.4 API Design

**Rust API Guidelines (official) checklist categories:**

**Naming (C-CASE, C-CONV, C-GETTER, C-ITER, C-ITER-TY):**
- `UpperCamelCase` for types/traits, `snake_case` for functions/variables
- Acronyms treated as single word: `Uuid` not `UUID`
- Getter methods named after field (no `get_` prefix): `fn name(&self) -> &str`
- Iterator-producing methods: `iter()`, `iter_mut()`, `into_iter()`
- Iterator types match method names: `fn iter(&self) -> Iter<'_>`

**Common Trait Implementations (C-COMMON-TRAITS):**
- Eagerly implement: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default`, `Display`
- Implement `Send` and `Sync` where possible (C-SEND-SYNC)
- Implement `From`/`Into` conversions (C-CONV-TRAITS)
- Collections implement `FromIterator` and `Extend` (C-COLLECT)
- Implement `Serialize`/`Deserialize` behind feature flags (C-SERDE)

**Error Types (C-GOOD-ERR):**
- Implement `std::error::Error`
- Meaningful, context-rich messages
- Errors are `Send + Sync + 'static`

**Type Safety (C-CUSTOM-TYPE, C-NEWTYPE, C-BITFLAG):**
- Newtypes to distinguish semantically different values of the same underlying type
- Builder pattern for complex construction
- Typestate pattern for compile-time state machine enforcement
- Make invalid states unrepresentable

```rust
// BAD: stringly-typed
fn create_user(name: &str, email: &str, role: &str) -> User { ... }

// GOOD: newtype wrappers
fn create_user(name: UserName, email: Email, role: Role) -> User { ... }
```

**Builder Pattern:**

```rust
// Good: builder with typestate for required fields
let config = Config::builder()
    .host("localhost")       // required
    .port(8080)              // required
    .timeout(Duration::from_secs(30)) // optional
    .build();                // only available when required fields set
```

### 2.5 Performance

**Check for:**

- **Clone abuse**: `.clone()` on `String`, `Vec<T>`, `HashMap` etc. when borrowing would work
- **Unnecessary allocations**:

  ```rust
  // BAD: allocates on every iteration
  for line in reader.lines() {
      let line = line?;
      process(&line);
  }

  // BETTER: reuse buffer
  let mut buf = String::new();
  while reader.read_line(&mut buf)? > 0 {
      process(&buf);
      buf.clear();
  }
  ```

- **Vec pre-allocation**: Use `Vec::with_capacity(n)` when size is known
- **String building**: Use `format!()` for simple cases, `String::with_capacity()` + `push_str()` for loops
- **Iterator misuse**: Prefer lazy iterators over collecting into intermediate `Vec`s

  ```rust
  // BAD: unnecessary intermediate collection
  let filtered: Vec<_> = items.iter().filter(|x| x.active).collect();
  let result: Vec<_> = filtered.iter().map(|x| x.name()).collect();

  // GOOD: chain iterators
  let result: Vec<_> = items.iter()
      .filter(|x| x.active)
      .map(|x| x.name())
      .collect();
  ```

- **`to_string()` vs `to_owned()`**: Use `.to_owned()` for `&str` -> `String` (avoids formatting machinery)
- **Box vs inline**: Small types don't benefit from boxing
- **Arc/Rc cycles**: Use `Weak` to break reference cycles

### 2.6 Concurrency and Async

**Check for:**

- **Blocking in async context**: Never call blocking I/O or `std::thread::sleep` in async code. Use `tokio::spawn_blocking` for blocking operations, `tokio::time::sleep` for delays.

  ```rust
  // BAD: blocks the executor thread
  async fn fetch_data() -> Result<Data> {
      let content = std::fs::read_to_string("data.json")?; // blocking!
      Ok(serde_json::from_str(&content)?)
  }

  // GOOD: use async fs or spawn_blocking
  async fn fetch_data() -> Result<Data> {
      let content = tokio::fs::read_to_string("data.json").await?;
      Ok(serde_json::from_str(&content)?)
  }
  ```

- **MutexGuard across `.await`**: Never hold a `MutexGuard` (std or tokio) across an `.await` point. Causes deadlocks with single-threaded runtimes, performance issues with multi-threaded.

  ```rust
  // BAD: guard held across await
  let guard = mutex.lock().await;
  do_async_work().await;  // guard still held!
  drop(guard);

  // GOOD: drop before await
  {
      let guard = mutex.lock().await;
      let value = guard.clone();
  } // guard dropped here
  do_async_work().await;
  ```

- **`Send + Sync` bounds**: Futures used with `tokio::spawn` must be `Send`. Check that types in async code are `Send` where needed.

- **Task cancellation safety**: Async code should be safe to cancel at any `.await` point. Resources should be cleaned up properly.

- **`std::sync::Mutex` vs `tokio::sync::Mutex`**: Prefer `std::sync::Mutex` for short critical sections (lower overhead). Use `tokio::sync::Mutex` only when you need to hold the guard across `.await`.

- **No async drop**: Rust has no `async Drop`. Use explicit cleanup methods for async resources.

### 2.7 Anti-Patterns

From the Rust Design Patterns catalog:

- **Deref polymorphism**: Using `Deref`/`DerefMut` to emulate inheritance. The compiler does not deref on trait dispatch, leading to subtle bugs.
- **Clone to satisfy borrow checker**: Sprinkling `.clone()` everywhere instead of restructuring ownership.
- **Stringly-typed APIs**: Using `String` where enums, newtypes, or specific types would be better.
- **God struct**: One struct that holds everything. Break into smaller, focused types.
- **`#[deny(warnings)]` in library code**: Breaks downstream users when new warnings are added. Use `#[warn(warnings)]` in libraries; `#[deny(warnings)]` only in CI.
- **Initializing objects after construction**: Use builders or constructors that produce fully-valid objects. Don't have `.init()` methods.

---

## 3. Clippy Lint Categories

### 3.1 Category Overview

| Category | Default Level | Purpose | Recommended Action |
|---|---|---|---|
| **correctness** | deny | Code that is outright wrong | Always fix |
| **suspicious** | warn | Code that is very likely wrong | Investigate all |
| **style** | warn | Idiomatic Rust conventions | Apply in new code |
| **complexity** | warn | Unnecessarily complex code | Simplify |
| **perf** | warn | Performance improvements | Apply unless readability trade-off |
| **pedantic** | allow | Opinionated, stricter checks | Enable selectively |
| **nursery** | allow | Experimental lints | Cherry-pick useful ones |
| **restriction** | allow | Situational restrictions | Never enable as group; pick individually |

### 3.2 Recommended Clippy Configuration

```toml
# In Cargo.toml or clippy.toml
[lints.clippy]
# Enable pedantic but allow specific noisy ones
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "warn"
missing_panics_doc = "warn"

# Key restriction lints to consider enabling:
# clone_on_ref_ptr, create_dir, dbg_macro, exit, expect_used,
# panic, print_stderr, print_stdout, todo, unimplemented, unwrap_used
```

### 3.3 Most Important Individual Lints for Review

**Correctness/Safety:**
- `invalid_reference_casting`, `transmute_undefined_repr`, `undropped_manually_drops`
- `mismatched_target_os`, `suspicious_else_formatting`

**Performance:**
- `needless_collect`, `redundant_allocation`, `box_collection`
- `manual_memcpy`, `slow_vector_initialization`
- `large_enum_variant` (consider boxing large variants)

**Style/Idiom:**
- `needless_return`, `redundant_closure`, `manual_map`
- `single_match` (use `if let`), `match_bool` (use `if`)

---

## 4. PR Review: Cargo.toml and Dependencies

### 4.1 Cargo.toml Review Checklist

- **Edition field**: Should be `edition = "2024"` for new projects
- **MSRV**: `rust-version` field set if targeting stability
- **Version**: Follows semver correctly; breaking changes = major bump
- **License**: `license` field present for published crates
- **Features**:
  - Default features are minimal and well-chosen
  - Optional dependencies gated behind features
  - `default-features = false` used on dependencies where appropriate
  - No circular feature dependencies
  - Features are additive only (enabling a feature never removes functionality)
- **Dependencies**:
  - Version requirements not overly broad (prefer `"1.2"` over `"1"` or `"*"`)
  - No pinned exact versions unless necessary (prefer `"~1.2.3"` if needed)
  - `workspace = true` used consistently in workspace members
  - Dev-dependencies separated from regular dependencies
  - Build-dependencies only where needed

### 4.2 Dependency Hygiene

**Tools to integrate:**
- **`cargo audit`**: Scans RustSec advisory database. Minimum required. Run in CI.
- **`cargo deny`**: Checks licenses, duplicate deps, banned crates, source policies. Recommended for all projects.
- **`cargo vet`**: Audit trail for third-party dependencies. Used by Mozilla and Google for supply chain security.
- **`cargo geiger`**: Reports unsafe code usage across all dependencies.
- **`cargo outdated`**: Identifies outdated dependencies.

**Review concerns:**
- New dependencies justified (not adding a crate for trivial functionality)
- Dependency count not excessive (each dep is attack surface and compile time)
- Yanked versions not referenced
- No `path` dependencies in published crates
- Lock file (`Cargo.lock`) committed for applications, not for libraries

### 4.3 Public API Surface

- **Visibility**: Only expose what is intended. Use `pub(crate)`, `pub(super)` liberally. `pub` items are commitments.
- **Re-exports**: Public re-exports form part of the API. Review carefully.
- **Type leakage**: Private types should not leak through public APIs.
- **Breaking changes**:
  - Adding required fields to public structs (use `#[non_exhaustive]`)
  - Removing public items
  - Changing function signatures
  - Adding required trait methods (provide defaults)
  - `#[non_exhaustive]` on public enums and structs to allow future extension

```rust
// GOOD: non_exhaustive allows adding variants later
#[non_exhaustive]
pub enum Error {
    NotFound,
    Unauthorized,
}

// GOOD: non_exhaustive allows adding fields later
#[non_exhaustive]
pub struct Config {
    pub host: String,
    pub port: u16,
}
```

---

## 5. Documentation Expectations

### 5.1 Doc Comment Standards

- **All public items** must have doc comments (`///`)
- **Crate-level docs** (`//!` at top of `lib.rs`) describing purpose, usage, and examples
- **Summary line**: First line is a single sentence, third-person singular present tense ("Returns the..." not "Return the...")
- **Standard sections** (in this order when applicable):
  - `# Examples` (plural, even if one example)
  - `# Errors` (when function returns `Result`)
  - `# Panics` (when function can panic)
  - `# Safety` (for `unsafe` functions)
- **Doc examples compile and run**: They are tested by `cargo test --doc`

```rust
/// Parses the configuration from the given TOML string.
///
/// # Examples
///
/// ```
/// let config = my_crate::parse_config("host = 'localhost'")?;
/// assert_eq!(config.host, "localhost");
/// # Ok::<(), my_crate::Error>(())
/// ```
///
/// # Errors
///
/// Returns `ConfigError::Parse` if the TOML is malformed.
/// Returns `ConfigError::Validation` if required fields are missing.
pub fn parse_config(toml: &str) -> Result<Config, ConfigError> { ... }
```

### 5.2 Internal Documentation

- Complex algorithms get `//` comments explaining the "why"
- Modules get `//!` doc comments explaining their role
- Non-obvious type parameters get explanatory comments
- `TODO`, `FIXME`, `HACK` comments tracked and addressed

---

## 6. Test Coverage Patterns

### 6.1 Test Organization

- **Unit tests**: In `#[cfg(test)] mod tests` within each source file. Test private interfaces, edge cases, error paths.
- **Integration tests**: In `tests/` directory. Test public API only, as an external consumer would.
- **Doc tests**: In doc comments. Serve as both documentation and tests.
- **Property-based tests**: Use `proptest` for core logic. Define properties that hold for all valid inputs.

### 6.2 What to Check in Test Review

- Happy path AND error paths tested
- Edge cases: empty input, zero values, maximum values, Unicode, concurrent access
- Error types returned correctly (not just "returns Err")
- No `unwrap()` in tests where the error message would be unhelpful (use `?` in test functions returning `Result`)
- Tests are deterministic (no reliance on timing, file system ordering, etc.)
- Mock/stub external services; don't make real HTTP calls in unit tests
- Async tests use `#[tokio::test]` not manual runtime setup
- Test names describe what is being tested: `test_parse_config_missing_host_returns_error`

### 6.3 Coverage Tools

- **`cargo-llvm-cov`**: LLVM-based source coverage. Recommended.
- **`cargo-tarpaulin`**: Alternative, particularly on Linux.
- Aim for high coverage on core business logic; 100% coverage is not a goal in itself.

---

## 7. Edition 2024 Specific Review Concerns

When reviewing code targeting edition 2024, additionally verify:

1. **No implicit unsafe in unsafe fns**: Every unsafe operation in an unsafe function has an explicit `unsafe {}` block with a `// SAFETY:` comment.
2. **No `static mut` references**: Use `Mutex`, `OnceLock`, atomics, or raw pointers.
3. **RPIT captures reviewed**: If functions return `impl Trait`, verify the captured lifetime set is intentional. Watch for accidental captures causing lifetime errors or over-constraining.
4. **Extern blocks are `unsafe extern`**: All `extern` blocks marked `unsafe`.
5. **Unsafe attributes properly marked**: `#[unsafe(no_mangle)]`, `#[unsafe(export_name)]`, `#[unsafe(link_section)]`.
6. **`gen` not used as identifier**: Reserved keyword.
7. **Macro fragment specifiers**: `expr` now matches `const` and `_`. Use `expr_2021` if old behavior needed.
8. **Prelude conflicts**: `Future` and `IntoFuture` in prelude may conflict with local traits.
9. **Tail expression temporaries**: Review code that relies on temporaries living until end of block.
10. **Match ergonomics**: No `ref`/`mut` in auto-deref patterns.

---

## Sources

- [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
- [Rust 1.85.0 Announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Rust Edition 2024 Annotated](https://bertptrs.nl/2025/02/23/rust-edition-2024-annotated.html)
- [Changes to impl Trait in Rust 2024](https://blog.rust-lang.org/2024/09/05/impl-trait-capture-rules/)
- [RPIT Lifetime Capture Rules RFC 3498](https://rust-lang.github.io/rfcs/3498-lifetime-capture-rules-2024.html)
- [Updating a Large Codebase to Rust 2024](https://codeandbitters.com/rust-2024-upgrade/)
- [Rust Code Review Checklist (Pull Panda)](https://pullpanda.io/blog/rust-code-review-checklist)
- [Microsoft Rust Guidelines Checklist](https://microsoft.github.io/rust-guidelines/guidelines/checklist/index.html)
- [Rust API Guidelines Checklist](https://rust-lang.github.io/api-guidelines/checklist.html)
- [Rust API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
- [Rust Design Patterns (Anti-patterns)](https://rust-unofficial.github.io/patterns/anti_patterns/)
- [Rust Security Best Practices 2025](https://corgea.com/Learn/rust-security-best-practices-2025)
- [Clippy Lint Categories](https://doc.rust-lang.org/stable/clippy/lints.html)
- [Clippy Lint Index](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Error Handling: thiserror, anyhow](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)
- [Error Handling Compared: anyhow vs thiserror vs snafu](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003)
- [Common Mistakes with Rust Async](https://www.qovery.com/blog/common-mistakes-with-rust-async)
- [Async Rust: Runtimes](https://corrode.dev/blog/async/)
- [Fuchsia Unsafe Code Guidelines](https://fuchsia.googlesource.com/fuchsia/+/master/docs/development/languages/rust/unsafe.md)
- [Rust Safety Comments Policy (std-dev-guide)](https://std-dev-guide.rust-lang.org/policy/safety-comments.html)
- [Comparing Rust Supply Chain Safety Tools](https://blog.logrocket.com/comparing-rust-supply-chain-safety-tools/)
- [cargo-vet (Mozilla)](https://github.com/mozilla/cargo-vet)
- [Rust Testing Best Practices](https://medium.com/@ashusk_1790/rust-testing-best-practices-unit-to-integration-965b39a8212f)
- [Defensive Programming in Rust](https://corrode.dev/blog/defensive-programming/)
- [Typestate Pattern in Rust](https://cliffle.com/blog/rust-typestate/)
- [Mastering Rust Newtypes](https://softwaremill.com/mastering-rust-patterns-vol-1-rust-newtypes/)
- [Idiomatic Rust](https://github.com/mre/idiomatic-rust)
- [Rust Perf Pitfalls](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)
- [Heap Allocations - Rust Performance Book](https://nnethercote.github.io/perf-book/heap-allocations.html)

## Related

- [[iterations/iteration-1-code-review]] — actual code review using these guidelines
