//! fixture-refresh — read-only API drift radar for recorded API responses vs.
//! hand-authored wiremock fixtures.
//!
//! Usage:
//!   fixture-refresh --recorded <DIR> [--fixtures <DIR>]
//!   fixture-refresh --recorded <DIR> --shape-report [--out <FILE>]
//!
//! Workflow:
//! 1. Run the live test suite with `HOPPY_RECORD_DIR=<scratch>` to capture fresh
//!    responses with auto-derived filenames like `core/GET_dnszone_50001.json`.
//! 2. Run `fixture-refresh --recorded <scratch>` (default mode) to preview byte
//!    drift between recordings and the checked-in fixtures.
//! 3. Run `fixture-refresh --recorded <scratch> --shape-report` for a markdown
//!    report of key/type drift plus a leak audit over the recordings.
//!
//! This tool never writes to `fixtures/`. Fixtures are test contracts —
//! updating them is a deliberate, reviewed change made inside an iteration
//! that also updates client types and tests, not a side effect of running a
//! recording sweep. See `hoppy-knowledgebase/decision-log.md`.
//!
//! Exit codes (only meaningful in `--shape-report` mode; default mode always
//! exits 0 on success):
//!   0 — no drift and no leak-audit hits
//!   1 — drift found (added/removed/type-changed keys, or unmapped/collisions)
//!   2 — leak-audit hits found (leaks take priority over drift)

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use analyzer::FixtureEntry;

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "fixture-refresh",
    about = "Read-only API drift radar: diff recorded API responses against hand-authored wiremock fixtures",
    long_about = "Read-only API drift radar: diff recorded API responses against hand-authored \
wiremock fixtures. Never writes to fixtures/.\n\n\
Exit codes (--shape-report only): 0 = clean, 1 = drift found, 2 = leak-audit hits."
)]
struct Cli {
    /// Directory of recording outputs (e.g. fixtures-recorded/ produced by HOPPY_RECORD_DIR=...)
    #[arg(long)]
    recorded: PathBuf,

    /// Root of the descriptive-name fixture tree to compare against (default: fixtures/)
    #[arg(long, default_value = "fixtures")]
    fixtures: PathBuf,

    /// Emit a markdown shape-diff + leak-audit report instead of the byte-drift listing.
    #[arg(long)]
    shape_report: bool,

    /// Write the --shape-report markdown to this file instead of stdout.
    #[arg(long, requires = "shape_report")]
    out: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let recorded_dir = cli
        .recorded
        .canonicalize()
        .with_context(|| format!("--recorded directory not found: {}", cli.recorded.display()))?;
    let fixtures_dir = cli
        .fixtures
        .canonicalize()
        .with_context(|| format!("--fixtures directory not found: {}", cli.fixtures.display()))?;

    // §1 — Build fixture→(method, path) table from test source files.
    let crates_root = find_crates_root(&fixtures_dir)?;
    let entries = analyzer::scan_test_files(&crates_root)?;
    eprintln!("Scanned {} fixture→mock mappings", entries.len());

    // §2 — Match recordings to descriptive fixtures.
    let recordings = collect_recordings(&recorded_dir)?;
    eprintln!("Found {} recording files", recordings.len());

    let matches = matcher::match_recordings(&entries, &recordings);

    if cli.shape_report {
        return run_shape_report(&cli, &fixtures_dir, &crates_root, &matches);
    }

    run_drift_listing(&fixtures_dir, &matches)
}

// ---------------------------------------------------------------------------
// Default mode — read-only byte-drift listing
// ---------------------------------------------------------------------------

fn run_drift_listing(fixtures_dir: &Path, matches: &[RecordingMatch]) -> Result<()> {
    let mut drifted = 0usize;
    let mut identical = 0usize;
    let mut collisions = 0usize;
    let mut unmapped = 0usize;

    for m in matches {
        match m {
            RecordingMatch::Mapped {
                fixture_rel,
                recording_abs,
                ..
            } => {
                let fixture_abs = fixtures_dir.join(fixture_rel);
                let rec_bytes = std::fs::read(recording_abs)
                    .with_context(|| format!("reading {}", recording_abs.display()))?;

                if fixture_abs.exists() {
                    let fix_bytes = std::fs::read(&fixture_abs)
                        .with_context(|| format!("reading {}", fixture_abs.display()))?;
                    if rec_bytes == fix_bytes {
                        identical += 1;
                        continue;
                    }
                    let delta =
                        (rec_bytes.len() as i64 - fix_bytes.len() as i64).unsigned_abs() as usize;
                    drifted += 1;
                    println!("drift:   {} (Δ {} bytes)", fixture_rel, delta);
                } else {
                    // Fixture doesn't exist on disk — count as drift (new file)
                    drifted += 1;
                    println!("drift (new): {} ({} bytes)", fixture_rel, rec_bytes.len());
                }
            }
            RecordingMatch::Collision {
                recording_rel,
                candidates,
                ..
            } => {
                collisions += 1;
                println!("collision: {} → [{}]", recording_rel, candidates.join(", "));
            }
            RecordingMatch::Unmapped { recording_rel, .. } => {
                unmapped += 1;
                println!("unmapped: {}", recording_rel);
            }
        }
    }

    eprintln!(
        "\nSummary: {} drifted, {} identical, {} collisions, {} unmapped",
        drifted, identical, collisions, unmapped
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// --shape-report mode
// ---------------------------------------------------------------------------

fn run_shape_report(
    cli: &Cli,
    fixtures_dir: &Path,
    workspace_root: &Path,
    matches: &[RecordingMatch],
) -> Result<()> {
    let leak_patterns = leak_audit::load_extra_patterns(workspace_root)?;

    let report = report::build_report(fixtures_dir, matches, &leak_patterns)?;
    let markdown = report.to_markdown();

    match &cli.out {
        Some(path) => {
            std::fs::write(path, &markdown)
                .with_context(|| format!("writing report to {}", path.display()))?;
            eprintln!("Report written to {}", path.display());
        }
        None => {
            println!("{markdown}");
        }
    }

    eprintln!(
        "\nSummary: {} leak hits, {} endpoints with drift, {} collisions, {} unmapped",
        report.leak_hits.len(),
        report.endpoints_with_drift(),
        report.collisions.len(),
        report.unmapped.len()
    );

    std::process::exit(report.exit_code());
}

// ---------------------------------------------------------------------------
// Match result types (shared between matcher and main)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RecordingMatch {
    /// Recording matched exactly one descriptive fixture.
    Mapped {
        fixture_rel: String,
        recording_rel: String,
        recording_abs: PathBuf,
    },
    /// Recording matched multiple descriptive fixtures (ambiguous — skip).
    Collision {
        recording_rel: String,
        recording_abs: PathBuf,
        candidates: Vec<String>,
    },
    /// No descriptive fixture maps to this recording.
    Unmapped {
        recording_rel: String,
        recording_abs: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Recording discovery
// ---------------------------------------------------------------------------

/// A recording file under `<recorded_root>/<domain>/<METHOD>_<segments>.json`.
#[derive(Debug, Clone)]
pub struct Recording {
    pub method: String,
    /// Reconstructed API path, e.g. "/dnszone/50001".
    pub path: String,
    /// Relative path for display, e.g. "core/GET_dnszone_50001.json".
    pub rel: String,
    pub abs: PathBuf,
}

fn collect_recordings(recorded_dir: &Path) -> Result<Vec<Recording>> {
    use walkdir::WalkDir;
    let mut out = Vec::new();
    for entry in WalkDir::new(recorded_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
    {
        let entry = entry
            .with_context(|| format!("walking recorded directory {}", recorded_dir.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !filename.ends_with(".json") {
            continue;
        }
        let Some(domain) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        else {
            continue;
        };
        // filename: <METHOD>_<segments...>.json
        // The recorder encodes path slashes as underscores, so we reverse that
        // here. Note: literal underscores in path segments are ambiguous with
        // encoded slashes. In practice, bunny.net API paths use hyphens and
        // digits only — no literal underscores — so the simple inverse is
        // correct for this codebase.
        let stem = filename.trim_end_matches(".json");
        let Some(underscore_pos) = stem.find('_') else {
            continue;
        };
        let method = stem[..underscore_pos].to_uppercase();
        let segments_part = &stem[underscore_pos + 1..];
        let path_str = if segments_part == "root" {
            "/".to_string()
        } else {
            format!("/{}", segments_part.replace('_', "/"))
        };
        let rel = format!("{}/{}", domain, filename);
        out.push(Recording {
            method,
            path: path_str,
            rel,
            abs: path.to_path_buf(),
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Walk up from fixtures_dir to find the workspace root (contains "crates/" directory).
fn find_crates_root(fixtures_dir: &Path) -> Result<PathBuf> {
    let mut dir = fixtures_dir.to_path_buf();
    loop {
        if dir.join("crates").is_dir() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => anyhow::bail!(
                "Could not find workspace root (directory containing 'crates/') \
                 starting from {}",
                fixtures_dir.display()
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Module: analyzer
// ---------------------------------------------------------------------------

mod analyzer {
    //! §1 — Static analysis of wiremock fixture references in test source files.
    //!
    //! Walks all `crates/**/tests/**/*.rs` files (skipping `target/`, `support/`
    //! directories, `live_*` files, and `mod.rs` files), parses them with `syn`,
    //! and extracts:
    //!
    //! - `include_str!("…/fixtures/<domain>/<name>")` → const name
    //! - `Mock::given(method("M")).and(path("P")).respond_with(…set_body_raw(BODY, …))`
    //!   chains capturing `(method, path, body_expr, status)`.
    //!
    //! The two are joined to produce a `Vec<FixtureEntry>`.

    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use syn::visit::Visit;
    use syn::{Expr, ExprCall, ExprMethodCall, File, Lit, LitStr};
    use walkdir::WalkDir;

    /// One row in the fixture→mock mapping table.
    #[derive(Debug, Clone)]
    pub struct FixtureEntry {
        /// Relative fixture path inside `fixtures/`, e.g. "core/dnszone_get.json".
        pub fixture_rel: String,
        /// HTTP method, uppercase.
        pub method: String,
        /// Path as used in `path("...")`, e.g. "/dnszone/50001".
        pub path: String,
        /// HTTP status code (used in unit tests to verify extraction; matcher filters by 2xx
        /// on the internal MockChain before constructing FixtureEntry).
        #[allow(dead_code)]
        pub status: u16,
    }

    pub fn scan_test_files(workspace_root: &Path) -> Result<Vec<FixtureEntry>> {
        let mut all_entries: Vec<FixtureEntry> = Vec::new();
        let test_files = collect_test_files(workspace_root)?;

        for file_path in &test_files {
            match analyze_file(file_path) {
                Ok(mut entries) => all_entries.append(&mut entries),
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", file_path.display());
                }
            }
        }

        // Deduplicate: same (fixture_rel, method, path) from multiple test functions.
        all_entries.sort_by(|a, b| {
            a.fixture_rel
                .cmp(&b.fixture_rel)
                .then(a.method.cmp(&b.method))
                .then(a.path.cmp(&b.path))
        });
        all_entries.dedup_by(|a, b| {
            a.fixture_rel == b.fixture_rel && a.method == b.method && a.path == b.path
        });

        Ok(all_entries)
    }

    fn collect_test_files(workspace_root: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let crates_dir = workspace_root.join("crates");
        for entry in WalkDir::new(&crates_dir).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git"
        }) {
            let entry = entry
                .with_context(|| format!("walking test sources under {}", crates_dir.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !name.ends_with(".rs") {
                continue;
            }
            // Must be inside a tests/ directory
            let path_str = path.to_string_lossy();
            if !path_str.contains("/tests/") && !path_str.contains("\\tests\\") {
                continue;
            }
            // Skip support/ directories (helper modules, not fixture tests)
            if path_str.contains("/support/") || path_str.contains("\\support\\") {
                continue;
            }
            // Skip live_* test files (they don't use wiremock)
            if name.starts_with("live_") {
                continue;
            }
            // Skip mod.rs (module declarations only)
            if name == "mod.rs" {
                continue;
            }
            files.push(path.to_path_buf());
        }
        Ok(files)
    }

    fn analyze_file(path: &Path) -> Result<Vec<FixtureEntry>> {
        let src =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let syntax: File =
            syn::parse_file(&src).with_context(|| format!("parsing {}", path.display()))?;

        let mut visitor = FileVisitor::default();
        visitor.visit_file(&syntax);
        Ok(visitor.build_entries())
    }

    // -----------------------------------------------------------------------
    // Syn visitor
    // -----------------------------------------------------------------------

    #[derive(Default)]
    struct FileVisitor {
        /// const ident → relative fixture path (e.g. "FIXTURE_GET" → "core/billing_get.json")
        const_to_fixture: HashMap<String, String>,
        /// Collected mock chains before joining with consts.
        mock_chains: Vec<MockChain>,
    }

    #[derive(Debug)]
    struct MockChain {
        method: String,
        path: String,
        /// Either a const ident name or a quoted literal "rel/path".
        body_expr: String,
        status: u16,
    }

    impl FileVisitor {
        fn build_entries(self) -> Vec<FixtureEntry> {
            let mut out = Vec::new();
            for chain in self.mock_chains {
                if chain.status < 200 || chain.status >= 300 {
                    continue;
                }
                let fixture_rel = if chain.body_expr.starts_with('"') {
                    // Literal from support::fixture("rel") — strip surrounding quotes
                    chain.body_expr.trim_matches('"').to_string()
                } else {
                    match self.const_to_fixture.get(&chain.body_expr) {
                        Some(rel) => rel.clone(),
                        None => continue,
                    }
                };
                out.push(FixtureEntry {
                    fixture_rel,
                    method: chain.method,
                    path: chain.path,
                    status: chain.status,
                });
            }
            out
        }
    }

    impl<'ast> Visit<'ast> for FileVisitor {
        /// Collect `const NAME: &str = include_str!("…/fixtures/<rel>");`
        fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
            if let Expr::Macro(m) = node.expr.as_ref()
                && m.mac.path.is_ident("include_str")
                && let Ok(lit) = m.mac.parse_body::<LitStr>()
                && let Some(rel) = extract_fixture_rel(&lit.value())
            {
                self.const_to_fixture.insert(node.ident.to_string(), rel);
            }
            syn::visit::visit_item_const(self, node);
        }

        /// Collect `Mock::given(method(M)).and(path(P)).respond_with(ResponseTemplate::new(S).set_body_raw(B, …))`
        fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
            if let Some(chain) = try_parse_mock_chain(node) {
                self.mock_chains.push(chain);
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    /// Extract relative fixture path from an include_str argument.
    /// Looks for the `.../fixtures/<rel>` pattern.
    pub(crate) fn extract_fixture_rel(path: &str) -> Option<String> {
        let normalized = path.replace('\\', "/");
        let marker = "/fixtures/";
        let pos = normalized.rfind(marker)?;
        let rel = &normalized[pos + marker.len()..];
        if rel.is_empty() {
            None
        } else {
            Some(rel.to_string())
        }
    }

    // -----------------------------------------------------------------------
    // Mock chain parser
    //
    // Shape A (bunny-net-api tests — const ref):
    //   Mock::given(method("GET"))
    //       .and(path("/billing"))
    //       .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "…"))
    //
    // Shape B (hoppy-cli e2e tests — support::fixture call):
    //   Mock::given(method("GET"))
    //       .and(path("/dnszone/50001"))
    //       .respond_with(ResponseTemplate::new(200).set_body_raw(
    //           support::fixture("core/dnszone_get.json"), "…"))
    // -----------------------------------------------------------------------

    fn try_parse_mock_chain(node: &ExprMethodCall) -> Option<MockChain> {
        if node.method != "respond_with" {
            return None;
        }
        let arg = node.args.first()?;
        let (status, body_expr) = parse_response_template(arg)?;
        let (method, path) = extract_method_and_path(&node.receiver)?;
        Some(MockChain {
            method,
            path,
            body_expr,
            status,
        })
    }

    fn parse_response_template(expr: &Expr) -> Option<(u16, String)> {
        let Expr::MethodCall(mc) = expr else {
            return None;
        };
        if mc.method != "set_body_raw" {
            return None;
        }
        let body_arg = mc.args.first()?;
        let body_expr = extract_body_expr(body_arg)?;
        let status = extract_status_from_expr(&mc.receiver)?;
        Some((status, body_expr))
    }

    fn extract_status_from_expr(expr: &Expr) -> Option<u16> {
        match expr {
            Expr::Call(c) => extract_status_from_new_call(c),
            Expr::MethodCall(mc) => extract_status_from_expr(&mc.receiver),
            _ => None,
        }
    }

    fn extract_status_from_new_call(call: &ExprCall) -> Option<u16> {
        let Expr::Path(p) = call.func.as_ref() else {
            return None;
        };
        if p.path.segments.last()?.ident != "new" {
            return None;
        }
        let arg = call.args.first()?;
        if let Expr::Lit(lit) = arg
            && let Lit::Int(n) = &lit.lit
        {
            return n.base10_parse::<u16>().ok();
        }
        None
    }

    fn extract_body_expr(expr: &Expr) -> Option<String> {
        match expr {
            Expr::Path(p) => {
                // Simple ident like FIXTURE_GET
                Some(p.path.get_ident()?.to_string())
            }
            Expr::Call(c) => {
                // support::fixture("rel") or fixture("rel")
                let Expr::Path(p) = c.func.as_ref() else {
                    return None;
                };
                if p.path.segments.last()?.ident != "fixture" {
                    return None;
                }
                let arg = c.args.first()?;
                if let Expr::Lit(lit) = arg
                    && let Lit::Str(s) = &lit.lit
                {
                    return Some(format!("\"{}\"", s.value()));
                }
                None
            }
            _ => None,
        }
    }

    fn extract_method_and_path(expr: &Expr) -> Option<(String, String)> {
        let mut method_val: Option<String> = None;
        let mut path_val: Option<String> = None;
        collect_matchers(expr, &mut method_val, &mut path_val);
        Some((method_val?, path_val?))
    }

    fn collect_matchers(
        expr: &Expr,
        method_val: &mut Option<String>,
        path_val: &mut Option<String>,
    ) {
        match expr {
            Expr::MethodCall(mc) => {
                if (mc.method == "and" || mc.method == "given")
                    && let Some(arg) = mc.args.first()
                {
                    check_matcher_arg(arg, method_val, path_val);
                }
                collect_matchers(&mc.receiver, method_val, path_val);
            }
            Expr::Call(c) => {
                if let Some(arg) = c.args.first() {
                    check_matcher_arg(arg, method_val, path_val);
                }
            }
            _ => {}
        }
    }

    fn check_matcher_arg(
        expr: &Expr,
        method_val: &mut Option<String>,
        path_val: &mut Option<String>,
    ) {
        let Expr::Call(c) = expr else { return };
        let Expr::Path(p) = c.func.as_ref() else {
            return;
        };
        let Some(last) = p.path.segments.last() else {
            return;
        };
        let fn_name = last.ident.to_string();
        let Some(arg) = c.args.first() else { return };

        if fn_name == "method" {
            if let Some(s) = extract_str_lit(arg) {
                *method_val = Some(s.to_uppercase());
            }
        } else if fn_name == "path"
            && let Some(s) = extract_str_lit(arg)
        {
            *path_val = Some(s);
        }
        // path_regex is not used in this codebase — skip.
    }

    fn extract_str_lit(expr: &Expr) -> Option<String> {
        if let Expr::Lit(lit) = expr
            && let Lit::Str(s) = &lit.lit
        {
            return Some(s.value());
        }
        None
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        const SHAPE_A: &str = r#"
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const FIXTURE_GET: &str = include_str!("../../../../../fixtures/core/billing_get.json");
const FIXTURE_UNAUTH: &str = include_str!("../../../../../fixtures/core/error_unauthorized.json");

#[tokio::test]
async fn get_billing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(FIXTURE_GET, "application/json"))
        .mount(&server)
        .await;
}

#[tokio::test]
async fn get_billing_unauth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(FIXTURE_UNAUTH, "application/json"))
        .mount(&server)
        .await;
}
"#;

        const SHAPE_B: &str = r#"
use super::support;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn dns_export() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001/export"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_raw(support::fixture("core/dnszone_export.txt"), "text/plain"),
        )
        .mount(&server)
        .await;
}

#[tokio::test]
async fn dns_get() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/dnszone/50001"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            support::fixture("core/dnszone_get.json"),
            "application/json",
        ))
        .mount(&server)
        .await;
}
"#;

        fn parse_source(src: &str) -> Vec<FixtureEntry> {
            let syntax = syn::parse_file(src).expect("parse failed");
            let mut visitor = FileVisitor::default();
            visitor.visit_file(&syntax);
            visitor.build_entries()
        }

        #[test]
        fn shape_a_extracts_const_ref() {
            let entries = parse_source(SHAPE_A);
            assert_eq!(
                entries.len(),
                1,
                "should extract exactly one 2xx entry; got: {entries:?}"
            );
            let e = &entries[0];
            assert_eq!(e.method, "GET");
            assert_eq!(e.path, "/billing");
            assert_eq!(e.fixture_rel, "core/billing_get.json");
            assert_eq!(e.status, 200);
        }

        #[test]
        fn shape_b_extracts_support_fixture_call() {
            let entries = parse_source(SHAPE_B);
            assert_eq!(entries.len(), 2, "got: {entries:?}");
            let export = entries.iter().find(|e| e.path == "/dnszone/50001/export");
            let get = entries.iter().find(|e| e.path == "/dnszone/50001");
            assert!(export.is_some(), "export entry missing");
            assert!(get.is_some(), "get entry missing");
            assert_eq!(export.unwrap().fixture_rel, "core/dnszone_export.txt");
            assert_eq!(get.unwrap().fixture_rel, "core/dnszone_get.json");
        }

        #[test]
        fn non_2xx_excluded() {
            let src = r#"
const FIXTURE_ERR: &str = include_str!("../../../../../fixtures/core/error_unauthorized.json");
async fn t() {
    Mock::given(method("GET"))
        .and(path("/billing"))
        .respond_with(ResponseTemplate::new(401).set_body_raw(FIXTURE_ERR, "application/json"))
        .mount(&server).await;
}
"#;
            let entries = parse_source(src);
            assert!(entries.is_empty(), "non-2xx should be excluded");
        }

        #[test]
        fn extract_fixture_rel_handles_various_paths() {
            assert_eq!(
                extract_fixture_rel("../../../../../fixtures/core/billing_get.json"),
                Some("core/billing_get.json".to_string())
            );
            assert_eq!(
                extract_fixture_rel("../../fixtures/stream/video_get.json"),
                Some("stream/video_get.json".to_string())
            );
            assert_eq!(extract_fixture_rel("no_fixtures_here.json"), None);
        }
    }
}

// ---------------------------------------------------------------------------
// Module: matcher
// ---------------------------------------------------------------------------

mod matcher {
    //! §2 — Match recorded filenames to descriptive fixture entries.
    //!
    //! Recording filenames follow: `<METHOD>_<segments>.json`
    //! where segments = path.trim_matches('/').replace('/', "_").
    //!
    //! Matching strategy (segment-by-segment):
    //! - Non-numeric segments must match exactly (case-insensitive).
    //! - Numeric segments in the recording match any value in the fixture path
    //!   at the same position (handles concrete IDs like 50001 vs 1001).
    //!
    //! Collision: multiple distinct descriptive fixtures map to the same recording
    //! → skip and report for human resolution.

    use std::collections::HashMap;

    use super::{FixtureEntry, Recording, RecordingMatch};

    pub fn match_recordings(
        entries: &[FixtureEntry],
        recordings: &[Recording],
    ) -> Vec<RecordingMatch> {
        // Index: (method, path_segments_normalised) → Vec<fixture_rel>
        let mut fixture_index: HashMap<(String, Vec<String>), Vec<String>> = HashMap::new();
        for entry in entries {
            let key = (entry.method.to_uppercase(), normalise_segments(&entry.path));
            fixture_index
                .entry(key)
                .or_default()
                .push(entry.fixture_rel.clone());
        }

        recordings
            .iter()
            .map(|rec| {
                let candidates = find_candidates(&fixture_index, &rec.method, &rec.path);

                // Deduplicate candidates
                let mut deduped = candidates;
                deduped.sort();
                deduped.dedup();

                match deduped.len() {
                    0 => RecordingMatch::Unmapped {
                        recording_rel: rec.rel.clone(),
                        recording_abs: rec.abs.clone(),
                    },
                    1 => RecordingMatch::Mapped {
                        fixture_rel: deduped.remove(0),
                        recording_rel: rec.rel.clone(),
                        recording_abs: rec.abs.clone(),
                    },
                    _ => RecordingMatch::Collision {
                        recording_rel: rec.rel.clone(),
                        recording_abs: rec.abs.clone(),
                        candidates: deduped,
                    },
                }
            })
            .collect()
    }

    fn normalise_segments(path: &str) -> Vec<String> {
        path.trim_matches('/')
            .split('/')
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Find descriptive fixtures whose (method, path-shape) could match this recording.
    ///
    /// A recording path uses concrete numeric IDs; fixture paths may have the same
    /// or different numeric IDs. We match segment-by-segment:
    /// - If the recording segment is numeric, it matches any fixture segment at
    ///   that position.
    /// - Otherwise, segments must match exactly (case-insensitive).
    fn find_candidates(
        index: &HashMap<(String, Vec<String>), Vec<String>>,
        method: &str,
        rec_path: &str,
    ) -> Vec<String> {
        let method_upper = method.to_uppercase();
        let rec_segs = normalise_segments(rec_path);

        let mut candidates = Vec::new();
        for ((m, fix_segs), fixtures) in index {
            if *m != method_upper {
                continue;
            }
            if fix_segs.len() != rec_segs.len() {
                continue;
            }
            let matches = rec_segs.iter().zip(fix_segs.iter()).all(|(r, f)| {
                // Numeric wildcard: if both sides are numeric they are considered
                // equivalent regardless of the concrete value (e.g. the fixture
                // was written with id 1001, recording used 9999 — same shape).
                // Requiring BOTH sides to be numeric prevents a recording segment
                // like "9999" from matching a fixture segment like "default".
                if is_numeric(r) && is_numeric(f) {
                    return true;
                }
                r == f
            });
            if matches {
                candidates.extend(fixtures.iter().cloned());
            }
        }
        candidates
    }

    fn is_numeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{FixtureEntry, Recording, RecordingMatch};
        use std::path::PathBuf;
        use tempfile::tempdir;

        fn entry(fixture_rel: &str, method: &str, path: &str) -> FixtureEntry {
            FixtureEntry {
                fixture_rel: fixture_rel.to_string(),
                method: method.to_string(),
                path: path.to_string(),
                status: 200,
            }
        }

        fn recording(domain: &str, method: &str, path: &str, abs: PathBuf) -> Recording {
            let segments = path.trim_matches('/').replace('/', "_");
            let filename = if segments.is_empty() {
                format!("{}_root.json", method.to_uppercase())
            } else {
                format!("{}_{}.json", method.to_uppercase(), segments)
            };
            Recording {
                method: method.to_uppercase(),
                path: path.to_string(),
                rel: format!("{}/{}", domain, filename),
                abs,
            }
        }

        #[test]
        fn exact_path_match() {
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_billing.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry("core/billing_get.json", "GET", "/billing")];
            let recordings = vec![recording("core", "GET", "/billing", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert_eq!(results.len(), 1);
            assert!(
                matches!(&results[0], RecordingMatch::Mapped { fixture_rel, .. } if fixture_rel == "core/billing_get.json"),
                "got: {:?}",
                results[0]
            );
        }

        #[test]
        fn numeric_segment_matches() {
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_dnszone_50001.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry("core/dnszone_get.json", "GET", "/dnszone/50001")];
            let recordings = vec![recording("core", "GET", "/dnszone/50001", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(
                matches!(&results[0], RecordingMatch::Mapped { fixture_rel, .. } if fixture_rel == "core/dnszone_get.json"),
                "got: {:?}",
                results[0]
            );
        }

        #[test]
        fn numeric_cross_id_match() {
            // Recording has id 9999 but fixture was written with id 1001 — should still match.
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_pullzone_9999.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry("core/pullzone_get.json", "GET", "/pullzone/1001")];
            let recordings = vec![recording("core", "GET", "/pullzone/9999", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(
                matches!(&results[0], RecordingMatch::Mapped { fixture_rel, .. } if fixture_rel == "core/pullzone_get.json"),
                "got: {:?}",
                results[0]
            );
        }

        #[test]
        fn collision_two_fixtures_same_path() {
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_pullzone.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![
                entry("core/pullzone_list_paginated.json", "GET", "/pullzone"),
                entry("core/pullzone_get_with_edgerules.json", "GET", "/pullzone"),
            ];
            let recordings = vec![recording("core", "GET", "/pullzone", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(
                matches!(&results[0], RecordingMatch::Collision { .. }),
                "expected collision, got: {:?}",
                results[0]
            );
        }

        #[test]
        fn unmapped_unknown_path() {
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_unknown.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry("core/billing_get.json", "GET", "/billing")];
            let recordings = vec![recording("core", "GET", "/unknown", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(matches!(&results[0], RecordingMatch::Unmapped { .. }));
        }

        #[test]
        fn method_mismatch_is_unmapped() {
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("POST_billing.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry("core/billing_get.json", "GET", "/billing")];
            let recordings = vec![recording("core", "POST", "/billing", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(matches!(&results[0], RecordingMatch::Unmapped { .. }));
        }

        #[test]
        fn numeric_recording_does_not_match_non_numeric_fixture_segment() {
            // Guard against one-sided wildcard: a recording with id 9999 must NOT
            // match a fixture whose corresponding segment is a non-numeric word like
            // "default", even though 9999 is numeric.
            let dir = tempdir().unwrap();
            let rec_path = dir.path().join("GET_pullzone_9999.json");
            std::fs::write(&rec_path, b"{}").unwrap();

            let entries = vec![entry(
                "core/pullzone_get_default.json",
                "GET",
                "/pullzone/default",
            )];
            let recordings = vec![recording("core", "GET", "/pullzone/9999", rec_path)];

            let results = match_recordings(&entries, &recordings);
            assert!(
                matches!(&results[0], RecordingMatch::Unmapped { .. }),
                "numeric recording segment should not match non-numeric fixture segment; got: {:?}",
                results[0]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Module: shape_diff
// ---------------------------------------------------------------------------

mod shape_diff {
    //! §shape-diff — key-path + type diff between two JSON documents.
    //!
    //! Compares a recorded response against a checked-in fixture and reports:
    //! - keys present in the recording but not the fixture ("added" — the API
    //!   grew a field the client doesn't model yet)
    //! - keys present in the fixture but not the recording ("removed" — the
    //!   client models a field the live API no longer returns)
    //! - keys present in both with a different JSON type ("type-changed")
    //!
    //! Array elements are collapsed to a single representative path
    //! (`Items.0.Name`) so an array of N objects doesn't produce N near-
    //! identical diagnostics. Noisy path segments (date-like map keys) are
    //! dropped by `noise::is_noisy_segment` before diffing.

    use std::collections::BTreeMap;

    use serde_json::Value;

    use super::noise;

    /// A JSON "type shape" coarse enough to catch drift (string→array etc.)
    /// without being sensitive to numeric width or exact null handling.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum JsonType {
        Null,
        Bool,
        Number,
        String,
        Array,
        Object,
    }

    impl JsonType {
        fn of(value: &Value) -> Self {
            match value {
                Value::Null => JsonType::Null,
                Value::Bool(_) => JsonType::Bool,
                Value::Number(_) => JsonType::Number,
                Value::String(_) => JsonType::String,
                Value::Array(_) => JsonType::Array,
                Value::Object(_) => JsonType::Object,
            }
        }
    }

    impl std::fmt::Display for JsonType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                JsonType::Null => "null",
                JsonType::Bool => "bool",
                JsonType::Number => "number",
                JsonType::String => "string",
                JsonType::Array => "array",
                JsonType::Object => "object",
            };
            f.write_str(s)
        }
    }

    /// Result of diffing two JSON documents by key path + type.
    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub struct ShapeDiff {
        /// Key paths present in the recording but not the fixture, sorted.
        pub added: Vec<String>,
        /// Key paths present in the fixture but not the recording, sorted.
        pub removed: Vec<String>,
        /// Key paths present in both with a different JSON type: (path, fixture_type, recording_type).
        pub type_changed: Vec<(String, JsonType, JsonType)>,
    }

    impl ShapeDiff {
        pub fn is_empty(&self) -> bool {
            self.added.is_empty() && self.removed.is_empty() && self.type_changed.is_empty()
        }
    }

    /// Diff `recording` against `fixture`, collecting a flat key-path → type map
    /// for each side (with array indices collapsed to `0` and noisy segments
    /// dropped), then comparing the two maps.
    pub fn diff(fixture: &Value, recording: &Value) -> ShapeDiff {
        let fixture_paths = flatten(fixture);
        let recording_paths = flatten(recording);

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut type_changed = Vec::new();

        for (path, rec_ty) in &recording_paths {
            match fixture_paths.get(path) {
                None => added.push(path.clone()),
                Some(fix_ty) if fix_ty != rec_ty => {
                    type_changed.push((path.clone(), *fix_ty, *rec_ty));
                }
                Some(_) => {}
            }
        }
        for path in fixture_paths.keys() {
            if !recording_paths.contains_key(path) {
                removed.push(path.clone());
            }
        }

        added.sort();
        added.dedup();
        removed.sort();
        removed.dedup();
        type_changed.sort();
        type_changed.dedup();

        ShapeDiff {
            added,
            removed,
            type_changed,
        }
    }

    /// Flatten a JSON value into a `path -> type` map. Array indices collapse
    /// to a single representative element (index `0` in the dot-joined path,
    /// e.g. `Items.0.Name`); noisy segments (date-like map keys) drop the
    /// whole path. Uses a `BTreeMap` so the "last type wins" collapse of
    /// array elements is deterministic regardless of array order.
    fn flatten(value: &Value) -> BTreeMap<String, JsonType> {
        let mut out = BTreeMap::new();
        flatten_into(value, String::new(), &mut out);
        out
    }

    fn flatten_into(value: &Value, prefix: String, out: &mut BTreeMap<String, JsonType>) {
        if !prefix.is_empty() {
            out.insert(prefix.clone(), JsonType::of(value));
        }
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    if noise::is_noisy_segment(key) {
                        continue;
                    }
                    let child_prefix = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}.{key}")
                    };
                    flatten_into(val, child_prefix, out);
                }
            }
            Value::Array(items) => {
                // Collapse all elements onto index 0 so an N-element array
                // produces one representative path, not N. Nested object keys
                // are unioned across elements (every child key seen anywhere
                // gets a path), but for the collapsed path's OWN type the last
                // element wins — a heterogeneous array mixing e.g. objects and
                // scalars reports only the final element's type. bunny.net
                // arrays are homogeneous by schema, so this is acceptable for
                // a diagnostic report. A no-op for an empty array.
                let child_prefix = if prefix.is_empty() {
                    "0".to_string()
                } else {
                    format!("{prefix}.0")
                };
                for item in items {
                    flatten_into(item, child_prefix.clone(), out);
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn detects_added_key() {
            let fixture = json!({ "Id": 1 });
            let recording = json!({ "Id": 1, "NewField": "x" });
            let d = diff(&fixture, &recording);
            assert_eq!(d.added, vec!["NewField".to_string()]);
            assert!(d.removed.is_empty());
            assert!(d.type_changed.is_empty());
        }

        #[test]
        fn detects_removed_key() {
            let fixture = json!({ "Id": 1, "Legacy": true });
            let recording = json!({ "Id": 1 });
            let d = diff(&fixture, &recording);
            assert_eq!(d.removed, vec!["Legacy".to_string()]);
            assert!(d.added.is_empty());
        }

        #[test]
        fn detects_type_change() {
            let fixture = json!({ "Tags": "single-tag" });
            let recording = json!({ "Tags": ["a", "b"] });
            let d = diff(&fixture, &recording);
            assert_eq!(
                d.type_changed,
                vec![("Tags".to_string(), JsonType::String, JsonType::Array)]
            );
        }

        #[test]
        fn identical_documents_produce_empty_diff() {
            let doc = json!({ "Id": 1, "Nested": { "A": [1, 2, 3] } });
            let d = diff(&doc, &doc);
            assert!(d.is_empty(), "expected empty diff, got {d:?}");
        }

        #[test]
        fn nested_added_key_reports_dotted_path() {
            let fixture = json!({ "Zone": { "Id": 1 } });
            let recording = json!({ "Zone": { "Id": 1, "Extra": "v" } });
            let d = diff(&fixture, &recording);
            assert_eq!(d.added, vec!["Zone.Extra".to_string()]);
        }

        #[test]
        fn array_of_objects_collapses_to_single_index() {
            // Ten items with the same shape must not produce ten paths.
            let items: Vec<_> = (0..10).map(|i| json!({ "Id": i, "Name": "x" })).collect();
            let fixture = json!({ "Items": items });
            let recording = json!({ "Items": items });
            let d = diff(&fixture, &recording);
            assert!(d.is_empty());
        }

        #[test]
        fn array_element_added_field_uses_representative_path() {
            let fixture = json!({ "Items": [{ "Id": 1 }, { "Id": 2 }] });
            let recording = json!({ "Items": [{ "Id": 1, "Extra": "v" }, { "Id": 2 }] });
            let d = diff(&fixture, &recording);
            assert_eq!(d.added, vec!["Items.0.Extra".to_string()]);
        }

        #[test]
        fn date_like_map_key_excluded_from_diff() {
            let fixture = json!({ "data": { "2026-03-01T00:00:00Z": { "count": 1 } } });
            let recording =
                json!({ "data": { "2026-03-02T00:00:00Z": { "count": 2, "extra": true } } });
            let d = diff(&fixture, &recording);
            assert!(
                d.is_empty(),
                "date-keyed chart entries should be filtered out entirely, got {d:?}"
            );
        }

        #[test]
        fn empty_array_produces_no_paths() {
            let fixture = json!({ "Items": [] });
            let recording = json!({ "Items": [] });
            let d = diff(&fixture, &recording);
            assert!(d.is_empty());
        }
    }
}

// ---------------------------------------------------------------------------
// Module: noise
// ---------------------------------------------------------------------------

mod noise {
    //! Noise filters shared by shape-diff: drop key-path segments that look
    //! like dates or timestamps rather than stable field names. Chart/metrics
    //! endpoints key their per-day buckets by date, so every recording sweep
    //! would otherwise report thousands of spurious "added" keys.

    /// Returns `true` when a single path segment (an object key, already
    /// split on `.`) looks like a date or timestamp rather than a field name.
    ///
    /// Recognised shapes (seen in real bunny.net chart payloads):
    /// - `DD-MM-YYYY`, e.g. `01-07-2026`
    /// - `YYYY-MM-DD`, e.g. `2026-07-01` (e.g. `compute/statistics.json`'s
    ///   `RequestsServedChart` / `TotalCpuTimeChart` per-day buckets)
    /// - ISO-8601 timestamps, e.g. `2026-07-10T00:00:00Z`, with or without
    ///   fractional seconds or a numeric UTC offset instead of `Z`.
    pub fn is_noisy_segment(segment: &str) -> bool {
        is_dd_mm_yyyy(segment) || is_yyyy_mm_dd(segment) || is_iso_timestamp(segment)
    }

    /// `DD-MM-YYYY`: dashes at positions 2 and 5 (e.g. `01-07-2026`).
    fn is_dd_mm_yyyy(s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() != 10 {
            return false;
        }
        let digit = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
        digit(0)
            && digit(1)
            && bytes[2] == b'-'
            && digit(3)
            && digit(4)
            && bytes[5] == b'-'
            && digit(6)
            && digit(7)
            && digit(8)
            && digit(9)
    }

    /// `YYYY-MM-DD`: dashes at positions 4 and 7 (e.g. `2026-07-01`).
    /// Structurally distinct from `DD-MM-YYYY` by dash position, so a
    /// 10-byte all-digit-plus-dashes segment matches exactly one of the two.
    fn is_yyyy_mm_dd(s: &str) -> bool {
        let bytes = s.as_bytes();
        if bytes.len() != 10 {
            return false;
        }
        let digit = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
        digit(0)
            && digit(1)
            && digit(2)
            && digit(3)
            && bytes[4] == b'-'
            && digit(5)
            && digit(6)
            && bytes[7] == b'-'
            && digit(8)
            && digit(9)
    }

    fn is_iso_timestamp(s: &str) -> bool {
        // YYYY-MM-DDTHH:MM:SS with optional fractional seconds and either a
        // literal 'Z' or a +HH:MM / -HH:MM offset. The tail is validated
        // exactly — a matching 19-byte prefix followed by arbitrary text
        // (e.g. "2026-07-10T00:00:00-some-slug") is NOT a timestamp, so a
        // real field name that merely starts date-like never gets filtered.
        let bytes = s.as_bytes();
        if bytes.len() < 19 {
            return false;
        }
        let digit = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
        let year_ok = digit(0) && digit(1) && digit(2) && digit(3);
        let month_ok = bytes.get(4) == Some(&b'-') && digit(5) && digit(6);
        let day_ok = bytes.get(7) == Some(&b'-') && digit(8) && digit(9);
        let sep_ok = matches!(bytes.get(10), Some(b'T') | Some(b't'));
        let hour_ok = digit(11) && digit(12);
        let min_ok = bytes.get(13) == Some(&b':') && digit(14) && digit(15);
        let sec_ok = bytes.get(16) == Some(&b':') && digit(17) && digit(18);
        if !(year_ok && month_ok && day_ok && sep_ok && hour_ok && min_ok && sec_ok) {
            return false;
        }
        is_valid_timestamp_tail(&bytes[19..])
    }

    /// Validates what may follow the `YYYY-MM-DDTHH:MM:SS` prefix: nothing,
    /// `Z`/`z`, a fractional-seconds part (`.` + digits), and/or a
    /// `+HH:MM` / `-HH:MM` offset.
    fn is_valid_timestamp_tail(mut tail: &[u8]) -> bool {
        if let [b'.', rest @ ..] = tail {
            let digits = rest.iter().take_while(|b| b.is_ascii_digit()).count();
            if digits == 0 {
                return false;
            }
            tail = &rest[digits..];
        }
        match tail {
            [] | [b'Z'] | [b'z'] => true,
            [b'+' | b'-', h1, h2, b':', m1, m2] => {
                [h1, h2, m1, m2].iter().all(|b| b.is_ascii_digit())
            }
            _ => false,
        }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn dd_mm_yyyy_is_noisy() {
            assert!(is_noisy_segment("01-07-2026"));
            assert!(is_noisy_segment("31-12-2025"));
        }

        #[test]
        fn yyyy_mm_dd_is_noisy() {
            // e.g. compute/statistics.json's RequestsServedChart keys.
            assert!(is_noisy_segment("2026-07-01"));
            assert!(is_noisy_segment("2024-01-01"));
        }

        #[test]
        fn iso_timestamp_is_noisy() {
            assert!(is_noisy_segment("2026-07-10T00:00:00Z"));
            assert!(is_noisy_segment("2026-07-10T00:00:00"));
            assert!(is_noisy_segment("2026-07-10T00:00:00.123Z"));
            assert!(is_noisy_segment("2026-07-10T00:00:00.2519408Z"));
            assert!(is_noisy_segment("2026-07-10T00:00:00+02:00"));
            assert!(is_noisy_segment("2026-07-10T00:00:00-05:00"));
        }

        #[test]
        fn timestamp_prefix_with_arbitrary_suffix_not_noisy() {
            // A valid 19-byte timestamp prefix followed by garbage is NOT a
            // timestamp — a field name that merely starts date-like must not
            // have its whole subtree dropped from the diff.
            assert!(!is_noisy_segment("2026-07-10T00:00:00-some-slug"));
            assert!(!is_noisy_segment("2026-07-10T00:00:00Zebra"));
            assert!(!is_noisy_segment("2026-07-10T00:00:00.Z"));
            assert!(!is_noisy_segment("2026-07-10T00:00:00+2:00"));
        }

        #[test]
        fn ordinary_field_names_not_noisy() {
            assert!(!is_noisy_segment("Id"));
            assert!(!is_noisy_segment("Name"));
            assert!(!is_noisy_segment("AllowedReferrers"));
            assert!(!is_noisy_segment("0"));
        }

        #[test]
        fn short_numeric_strings_not_noisy() {
            // Must not false-positive on plain numeric-ish identifiers.
            assert!(!is_noisy_segment("2026"));
            assert!(!is_noisy_segment("50001"));
        }

        #[test]
        fn version_string_not_noisy() {
            assert!(!is_noisy_segment("1.2.3"));
        }
    }
}

// ---------------------------------------------------------------------------
// Module: leak_audit
// ---------------------------------------------------------------------------

mod leak_audit {
    //! §3 — Leak audit: scan recording JSON for values that look like they
    //! should have been redacted before landing on disk.
    //!
    //! Three built-in rules plus an optional user-supplied pattern file:
    //! - Email-shaped strings outside `example.com` / `example.org` and not
    //!   the literal `<redacted>` sentinel.
    //! - 72-char double-UUID strings (the bunny.net account API key shape —
    //!   two concatenated 8-4-4-4-12 hex UUIDs). Re-implemented here (not
    //!   imported) because `bunny_net_api::recording::redact` is a private
    //!   module of the `bunny-net-api` crate; see
    //!   `crates/bunny-net-api/src/recording/redact.rs::is_account_api_key`
    //!   for the canonical version this mirrors.
    //! - String values under secret-ish key names (`*key*`, `*token*`,
    //!   `*password*`, `*secret*`, case-insensitive) that aren't
    //!   `<redacted>`, null, or empty.
    //!
    //! Tuned exclusions (so running this over the checked-in `fixtures/` tree
    //! as a sanity check produces zero false alarms):
    //! - `errorKey` (bunny.net error envelope field name, e.g.
    //!   `"errorKey": "invalid_plan_type"` — a machine-readable error code,
    //!   not a secret).
    //! - `KeyId` (an identifier *for* a key, not the key material itself).

    use std::path::Path;

    use anyhow::{Context, Result};
    use regex::Regex;
    use serde_json::Value;

    /// A single leak-audit hit: where it was found and what fired.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LeakHit {
        /// Relative recording path, e.g. "core/GET_billing.json".
        pub recording_rel: String,
        /// Dot-joined key path inside the JSON document where the value was found.
        pub key_path: String,
        /// Human-readable description of which rule fired.
        pub rule: String,
    }

    /// Key-name substrings (case-insensitive) that mark a field as secret-ish.
    const SECRET_KEY_PATTERNS: &[&str] = &["key", "token", "password", "secret"];

    /// Exact key names (case-insensitive) excluded from the secret-ish-key
    /// rule even though they match a substring pattern above — tuned against
    /// the real fixtures/ corpus to avoid false alarms.
    const SECRET_KEY_EXCLUSIONS: &[&str] = &[
        // bunny.net machine-readable error code, e.g. "invalid_plan_type" —
        // not a secret despite containing "key".
        "errorkey",
        // Identifier *for* a key (e.g. an access-key row id), not key material.
        "keyid",
        // Video library player UI color, e.g. "#ff0000" — not key material.
        "playerkeycolor",
        // DNSSEC public key — public by definition.
        "publickey",
    ];

    pub fn is_secret_key_name(key: &str) -> bool {
        let lower = key.to_lowercase();
        if SECRET_KEY_EXCLUSIONS.contains(&lower.as_str()) {
            return false;
        }
        SECRET_KEY_PATTERNS.iter().any(|pat| lower.contains(pat))
    }

    /// bunny.net account API keys are two concatenated UUIDs (72 chars).
    /// Mirrors `bunny_net_api::recording::redact::is_account_api_key`.
    pub fn is_double_uuid(s: &str) -> bool {
        let bytes = s.as_bytes();
        bytes.len() == 72 && is_uuid(&bytes[..36]) && is_uuid(&bytes[36..])
    }

    fn is_uuid(bytes: &[u8]) -> bool {
        bytes.len() == 36
            && bytes.iter().enumerate().all(|(i, b)| match i {
                8 | 13 | 18 | 23 => *b == b'-',
                _ => b.is_ascii_hexdigit(),
            })
    }

    /// Very small email matcher: local@domain.tld with no whitespace. Good
    /// enough to catch real leaked addresses without pulling in a full RFC
    /// 5322 parser.
    fn email_domain(s: &str) -> Option<&str> {
        let s = s.trim();
        if s.contains(char::is_whitespace) {
            return None;
        }
        let at = s.find('@')?;
        let (local, rest) = (&s[..at], &s[at + 1..]);
        if local.is_empty() || rest.is_empty() {
            return None;
        }
        let dot = rest.rfind('.')?;
        if dot == 0 || dot == rest.len() - 1 {
            return None;
        }
        Some(rest)
    }

    fn is_allowed_email_domain(domain: &str) -> bool {
        let lower = domain.to_lowercase();
        lower == "example.com"
            || lower == "example.org"
            || lower.ends_with(".example.com")
            || lower.ends_with(".example.org")
    }

    /// Load optional extra leak patterns from `<workspace_root>/.hoppy-leak-patterns`.
    /// One regex per line; blank lines and lines starting with `#` are ignored.
    /// A missing file is not an error — it just means no extra patterns apply.
    pub fn load_extra_patterns(workspace_root: &Path) -> Result<Vec<Regex>> {
        let path = workspace_root.join(".hoppy-leak-patterns");
        if !path.exists() {
            return Ok(Vec::new());
        }
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let mut patterns = Vec::new();
        for (lineno, line) in contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let re = Regex::new(trimmed).with_context(|| {
                format!(
                    "{}:{}: invalid regex: {trimmed:?}",
                    path.display(),
                    lineno + 1
                )
            })?;
            patterns.push(re);
        }
        Ok(patterns)
    }

    /// Scan a recording's parsed JSON value for leak-audit hits.
    pub fn scan(recording_rel: &str, value: &Value, extra_patterns: &[Regex]) -> Vec<LeakHit> {
        let mut hits = Vec::new();
        scan_value(
            recording_rel,
            value,
            String::new(),
            extra_patterns,
            &mut hits,
        );
        hits
    }

    fn scan_value(
        recording_rel: &str,
        value: &Value,
        path: String,
        extra_patterns: &[Regex],
        hits: &mut Vec<LeakHit>,
    ) {
        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    // Redaction leaves `"<redacted>"` for strings and `0` for
                    // numbers — anything else under a secret-ish key is a
                    // potential leak (bools/objects/arrays carry no secret
                    // material themselves; their leaves are scanned below).
                    let leaky_value = match val {
                        Value::String(s) => !is_redacted_or_empty(s),
                        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
                        _ => false,
                    };
                    if is_secret_key_name(key) && leaky_value {
                        hits.push(LeakHit {
                            recording_rel: recording_rel.to_string(),
                            key_path: child_path.clone(),
                            rule: "secret-ish key name".to_string(),
                        });
                    }
                    scan_value(recording_rel, val, child_path, extra_patterns, hits);
                }
            }
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    let child_path = if path.is_empty() {
                        i.to_string()
                    } else {
                        format!("{path}.{i}")
                    };
                    scan_value(recording_rel, item, child_path, extra_patterns, hits);
                }
            }
            Value::String(s) => {
                if !is_redacted_or_empty(s) {
                    if is_double_uuid(s) {
                        hits.push(LeakHit {
                            recording_rel: recording_rel.to_string(),
                            key_path: path.clone(),
                            rule: "double-UUID account API key shape".to_string(),
                        });
                    }
                    if let Some(domain) = email_domain(s)
                        && !is_allowed_email_domain(domain)
                    {
                        hits.push(LeakHit {
                            recording_rel: recording_rel.to_string(),
                            key_path: path.clone(),
                            rule: format!("email address (domain {domain})"),
                        });
                    }
                }
                for re in extra_patterns {
                    if re.is_match(s) {
                        hits.push(LeakHit {
                            recording_rel: recording_rel.to_string(),
                            key_path: path.clone(),
                            rule: format!(".hoppy-leak-patterns: /{}/", re.as_str()),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    fn is_redacted_or_empty(s: &str) -> bool {
        s.is_empty() || s == "<redacted>"
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use serde_json::json;

        #[test]
        fn flags_real_email_domain() {
            let v = json!({ "AuthorEmail": "jane@real-company.com" });
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 1);
            assert!(hits[0].rule.contains("email"));
        }

        #[test]
        fn allows_example_com_email() {
            let v = json!({ "ContactEmail": "user@example.com" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "got: {hits:?}");
        }

        #[test]
        fn allows_example_org_email() {
            let v = json!({ "ContactEmail": "user@example.org" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "got: {hits:?}");
        }

        #[test]
        fn allows_redacted_sentinel() {
            let v = json!({ "AuthorEmail": "<redacted>", "ApiKey": "<redacted>" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "got: {hits:?}");
        }

        #[test]
        fn flags_double_uuid_value() {
            let key = "eda66cfe-8fd7-4040-997f-77a6c66fe488ea41a773-201d-4cbf-81df-1735d605b486";
            let v = json!({ "SomeHarmlessField": key });
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 1);
            assert!(hits[0].rule.contains("double-UUID"));
        }

        #[test]
        fn single_uuid_not_flagged() {
            let v = json!({ "guid": "7ddb2cac-63f5-46c0-beed-f6566e0f6a07" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "single UUIDs are identifiers, not keys");
        }

        #[test]
        fn flags_secret_ish_key_name() {
            let v = json!({ "ApiKey": "sk_live_abc123", "Password": "hunter2", "Token": "tok_x" });
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 3, "got: {hits:?}");
        }

        #[test]
        fn numeric_value_under_secret_key_flagged_unless_zero() {
            // Redaction turns numbers under sensitive keys into 0 — any other
            // number under a secret-ish key escaped redaction.
            let v = json!({ "ApiKey": 123456, "Token": 0 });
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 1, "got: {hits:?}");
            assert_eq!(hits[0].key_path, "ApiKey");
        }

        #[test]
        fn error_key_field_not_flagged() {
            let v = json!({ "errorKey": "invalid_plan_type" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(
                hits.is_empty(),
                "errorKey is a machine-readable error code, not a secret"
            );
        }

        #[test]
        fn key_id_field_not_flagged() {
            let v = json!({ "KeyId": "42" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(
                hits.is_empty(),
                "KeyId identifies a key, it isn't key material"
            );
        }

        #[test]
        fn player_key_color_and_public_key_not_flagged() {
            // Found live during iter-78 dogfooding: a player UI color and a
            // DNSSEC public key are not secrets.
            let v = json!({ "PlayerKeyColor": "#ff0000", "PublicKey": "257 3 13 abc=" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "got: {hits:?}");
        }

        #[test]
        fn null_and_empty_secret_values_not_flagged() {
            let v = json!({ "ApiKey": null, "Token": "" });
            let hits = scan("core/x.json", &v, &[]);
            assert!(hits.is_empty(), "got: {hits:?}");
        }

        #[test]
        fn nested_object_scanned_for_leaks() {
            let v = json!({ "Zone": { "Owner": { "Email": "leaked@real.com" } } });
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].key_path, "Zone.Owner.Email");
        }

        #[test]
        fn array_of_objects_scanned_for_leaks() {
            let v = json!([{ "Email": "a@real.com" }, { "Email": "user@example.com" }]);
            let hits = scan("core/x.json", &v, &[]);
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].key_path, "0.Email");
        }

        #[test]
        fn extra_pattern_from_file_matches() {
            let re = Regex::new("hpmc-[a-z0-9-]+").unwrap();
            let v = json!({ "Name": "hpmc-realaccount-prod" });
            let hits = scan("core/x.json", &v, std::slice::from_ref(&re));
            assert_eq!(hits.len(), 1);
            assert!(hits[0].rule.contains("hoppy-leak-patterns"));
        }

        #[test]
        fn load_extra_patterns_missing_file_is_empty() {
            let dir = tempfile::tempdir().unwrap();
            let patterns = load_extra_patterns(dir.path()).unwrap();
            assert!(patterns.is_empty());
        }

        #[test]
        fn load_extra_patterns_parses_file_skipping_comments_and_blanks() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join(".hoppy-leak-patterns"),
                "# a comment\n\nhpmc-[a-z0-9-]+\n  \nfoo-\\d+\n",
            )
            .unwrap();
            let patterns = load_extra_patterns(dir.path()).unwrap();
            assert_eq!(patterns.len(), 2);
        }
    }
}

// ---------------------------------------------------------------------------
// Module: report
// ---------------------------------------------------------------------------

mod report {
    //! §report — assemble the `--shape-report` markdown document from the
    //! recording↔fixture matches: leak audit (top, most prominent), then
    //! per-domain / per-endpoint shape diffs, then unmapped + collision
    //! listings.

    use std::collections::BTreeMap;
    use std::path::Path;

    use anyhow::{Context, Result};
    use regex::Regex;
    use serde_json::Value;

    use super::RecordingMatch;
    use super::leak_audit::{self, LeakHit};
    use super::shape_diff::{self, ShapeDiff};

    pub struct Report {
        /// Leak-audit hits across all recordings, in scan order.
        pub leak_hits: Vec<LeakHit>,
        /// Per-fixture shape diff, keyed by fixture_rel, non-empty diffs only.
        pub drift: BTreeMap<String, ShapeDiff>,
        pub collisions: Vec<(String, Vec<String>)>,
        pub unmapped: Vec<String>,
    }

    impl Report {
        pub fn endpoints_with_drift(&self) -> usize {
            self.drift.len()
        }

        /// Exit code per the documented contract: 0 clean, 1 drift, 2 leaks.
        /// Leaks take priority over drift when both are present.
        pub fn exit_code(&self) -> i32 {
            if !self.leak_hits.is_empty() {
                2
            } else if !self.drift.is_empty()
                || !self.collisions.is_empty()
                || !self.unmapped.is_empty()
            {
                1
            } else {
                0
            }
        }

        pub fn to_markdown(&self) -> String {
            let mut out = String::new();
            out.push_str("# API drift radar report\n\n");

            // Leak audit is the most prominent section — it goes first.
            out.push_str("## Leak audit\n\n");
            if self.leak_hits.is_empty() {
                out.push_str("No leak-audit hits.\n\n");
            } else {
                out.push_str(&format!(
                    "**{} potential leak(s) found in recordings.** Do not commit these recordings.\n\n",
                    self.leak_hits.len()
                ));
                for hit in &self.leak_hits {
                    out.push_str(&format!(
                        "- `{}` at `{}` — {}\n",
                        hit.recording_rel, hit.key_path, hit.rule
                    ));
                }
                out.push('\n');
            }

            // Shape drift grouped by domain (first path segment of fixture_rel), then endpoint.
            out.push_str("## Shape drift\n\n");
            if self.drift.is_empty() {
                out.push_str("No key/type drift detected.\n\n");
            } else {
                let mut by_domain: BTreeMap<&str, Vec<(&str, &ShapeDiff)>> = BTreeMap::new();
                for (fixture_rel, diff) in &self.drift {
                    let domain = fixture_rel.split('/').next().unwrap_or(fixture_rel);
                    by_domain
                        .entry(domain)
                        .or_default()
                        .push((fixture_rel.as_str(), diff));
                }
                for (domain, endpoints) in &by_domain {
                    out.push_str(&format!("### {domain}\n\n"));
                    for (fixture_rel, diff) in endpoints {
                        out.push_str(&format!("#### {fixture_rel}\n\n"));
                        if !diff.added.is_empty() {
                            out.push_str("Added (in recording, not in fixture):\n\n");
                            for key in &diff.added {
                                out.push_str(&format!("- `{key}`\n"));
                            }
                            out.push('\n');
                        }
                        if !diff.removed.is_empty() {
                            out.push_str("Removed (in fixture, not in recording):\n\n");
                            for key in &diff.removed {
                                out.push_str(&format!("- `{key}`\n"));
                            }
                            out.push('\n');
                        }
                        if !diff.type_changed.is_empty() {
                            out.push_str("Type changed:\n\n");
                            for (key, fix_ty, rec_ty) in &diff.type_changed {
                                out.push_str(&format!("- `{key}`: {fix_ty} → {rec_ty}\n"));
                            }
                            out.push('\n');
                        }
                    }
                }
            }

            out.push_str("## Unmapped recordings\n\n");
            if self.unmapped.is_empty() {
                out.push_str("None.\n\n");
            } else {
                for rec in &self.unmapped {
                    out.push_str(&format!("- {rec}\n"));
                }
                out.push('\n');
            }

            out.push_str("## Collisions\n\n");
            if self.collisions.is_empty() {
                out.push_str("None.\n\n");
            } else {
                for (rec, candidates) in &self.collisions {
                    out.push_str(&format!("- {rec} → [{}]\n", candidates.join(", ")));
                }
                out.push('\n');
            }

            out
        }
    }

    pub fn build_report(
        fixtures_dir: &Path,
        matches: &[RecordingMatch],
        extra_leak_patterns: &[Regex],
    ) -> Result<Report> {
        let mut leak_hits = Vec::new();
        let mut drift = BTreeMap::new();
        let mut collisions = Vec::new();
        let mut unmapped = Vec::new();

        for m in matches {
            match m {
                RecordingMatch::Mapped {
                    fixture_rel,
                    recording_rel,
                    recording_abs,
                } => {
                    let rec_value = read_json(recording_abs)?;

                    // Leak hits are labelled with the RECORDING path (the file
                    // to locate/delete), not the mapped fixture name.
                    leak_hits.extend(leak_audit::scan(
                        recording_rel,
                        &rec_value,
                        extra_leak_patterns,
                    ));

                    let fixture_abs = fixtures_dir.join(fixture_rel);
                    if fixture_abs.exists() {
                        // Non-JSON fixtures (e.g. the DNS export .txt fixture)
                        // can't be shape-diffed — skip them for the diff, but
                        // they were still leak-scanned above via the recording.
                        if let Ok(fix_value) = read_json(&fixture_abs) {
                            let d = shape_diff::diff(&fix_value, &rec_value);
                            if !d.is_empty() {
                                drift.insert(fixture_rel.clone(), d);
                            }
                        }
                    } else {
                        // No fixture on disk yet — treat every top-level key as "added".
                        let d = shape_diff::diff(&Value::Null, &rec_value);
                        if !d.is_empty() {
                            drift.insert(fixture_rel.clone(), d);
                        }
                    }
                }
                // Collision/unmapped recordings have no fixture to diff, but
                // they can still carry leaks — scan them too.
                RecordingMatch::Collision {
                    recording_rel,
                    recording_abs,
                    candidates,
                } => {
                    let rec_value = read_json(recording_abs)?;
                    leak_hits.extend(leak_audit::scan(
                        recording_rel,
                        &rec_value,
                        extra_leak_patterns,
                    ));
                    collisions.push((recording_rel.clone(), candidates.clone()));
                }
                RecordingMatch::Unmapped {
                    recording_rel,
                    recording_abs,
                } => {
                    let rec_value = read_json(recording_abs)?;
                    leak_hits.extend(leak_audit::scan(
                        recording_rel,
                        &rec_value,
                        extra_leak_patterns,
                    ));
                    unmapped.push(recording_rel.clone());
                }
            }
        }

        Ok(Report {
            leak_hits,
            drift,
            collisions,
            unmapped,
        })
    }

    fn read_json(path: &Path) -> Result<Value> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing JSON from {}", path.display()))
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::PathBuf;
        use tempfile::tempdir;

        #[test]
        fn exit_code_clean() {
            let report = Report {
                leak_hits: vec![],
                drift: BTreeMap::new(),
                collisions: vec![],
                unmapped: vec![],
            };
            assert_eq!(report.exit_code(), 0);
        }

        #[test]
        fn exit_code_drift_only() {
            let mut drift = BTreeMap::new();
            drift.insert(
                "core/x.json".to_string(),
                ShapeDiff {
                    added: vec!["Foo".to_string()],
                    ..Default::default()
                },
            );
            let report = Report {
                leak_hits: vec![],
                drift,
                collisions: vec![],
                unmapped: vec![],
            };
            assert_eq!(report.exit_code(), 1);
        }

        #[test]
        fn exit_code_leak_takes_priority_over_drift() {
            let mut drift = BTreeMap::new();
            drift.insert(
                "core/x.json".to_string(),
                ShapeDiff {
                    added: vec!["Foo".to_string()],
                    ..Default::default()
                },
            );
            let report = Report {
                leak_hits: vec![LeakHit {
                    recording_rel: "core/x.json".to_string(),
                    key_path: "Email".to_string(),
                    rule: "email address".to_string(),
                }],
                drift,
                collisions: vec![],
                unmapped: vec![],
            };
            assert_eq!(report.exit_code(), 2);
        }

        #[test]
        fn exit_code_unmapped_counts_as_drift_signal() {
            let report = Report {
                leak_hits: vec![],
                drift: BTreeMap::new(),
                collisions: vec![],
                unmapped: vec!["core/GET_unknown.json".to_string()],
            };
            assert_eq!(report.exit_code(), 1);
        }

        #[test]
        fn markdown_contains_expected_sections() {
            let report = Report {
                leak_hits: vec![],
                drift: BTreeMap::new(),
                collisions: vec![],
                unmapped: vec![],
            };
            let md = report.to_markdown();
            assert!(md.contains("## Leak audit"));
            assert!(md.contains("## Shape drift"));
            assert!(md.contains("## Unmapped recordings"));
            assert!(md.contains("## Collisions"));
        }

        #[test]
        fn build_report_diffs_mapped_json_fixture() {
            let dir = tempdir().unwrap();
            let fixtures_dir = dir.path().join("fixtures");
            std::fs::create_dir_all(fixtures_dir.join("core")).unwrap();
            std::fs::write(fixtures_dir.join("core/billing_get.json"), r#"{"Id":1}"#).unwrap();

            let recorded_dir = dir.path().join("recorded");
            std::fs::create_dir_all(&recorded_dir).unwrap();
            let rec_path = recorded_dir.join("GET_billing.json");
            std::fs::write(&rec_path, r#"{"Id":1,"NewField":"x"}"#).unwrap();

            let matches = vec![RecordingMatch::Mapped {
                fixture_rel: "core/billing_get.json".to_string(),
                recording_rel: "core/GET_billing.json".to_string(),
                recording_abs: rec_path,
            }];

            let report = build_report(&fixtures_dir, &matches, &[]).unwrap();
            assert_eq!(report.drift.len(), 1);
            let diff = &report.drift["core/billing_get.json"];
            assert_eq!(diff.added, vec!["NewField".to_string()]);
        }

        #[test]
        fn build_report_scans_recordings_for_leaks() {
            let dir = tempdir().unwrap();
            let fixtures_dir = dir.path().join("fixtures");
            std::fs::create_dir_all(fixtures_dir.join("core")).unwrap();
            std::fs::write(fixtures_dir.join("core/billing_get.json"), r#"{"Id":1}"#).unwrap();

            let recorded_dir = dir.path().join("recorded");
            std::fs::create_dir_all(&recorded_dir).unwrap();
            let rec_path: PathBuf = recorded_dir.join("GET_billing.json");
            std::fs::write(&rec_path, r#"{"Id":1,"AuthorEmail":"leak@real.com"}"#).unwrap();

            let matches = vec![RecordingMatch::Mapped {
                fixture_rel: "core/billing_get.json".to_string(),
                recording_rel: "core/GET_billing.json".to_string(),
                recording_abs: rec_path,
            }];

            let report = build_report(&fixtures_dir, &matches, &[]).unwrap();
            assert_eq!(report.leak_hits.len(), 1);
            assert_eq!(
                report.leak_hits[0].recording_rel, "core/GET_billing.json",
                "leak hits must point at the recording file, not the mapped fixture"
            );
            assert_eq!(report.exit_code(), 2);
        }

        #[test]
        fn build_report_scans_unmapped_and_collision_recordings_for_leaks() {
            let dir = tempdir().unwrap();
            let fixtures_dir = dir.path().join("fixtures");
            std::fs::create_dir_all(&fixtures_dir).unwrap();

            let recorded_dir = dir.path().join("recorded");
            std::fs::create_dir_all(&recorded_dir).unwrap();
            let unmapped_path = recorded_dir.join("GET_new_endpoint.json");
            std::fs::write(&unmapped_path, r#"{"AuthorEmail":"leak@real.com"}"#).unwrap();
            let collision_path = recorded_dir.join("GET_ambiguous.json");
            std::fs::write(&collision_path, r#"{"ApiKey":"raw-secret"}"#).unwrap();

            let matches = vec![
                RecordingMatch::Unmapped {
                    recording_rel: "core/GET_new_endpoint.json".to_string(),
                    recording_abs: unmapped_path,
                },
                RecordingMatch::Collision {
                    recording_rel: "core/GET_ambiguous.json".to_string(),
                    recording_abs: collision_path,
                    candidates: vec!["core/a.json".to_string(), "core/b.json".to_string()],
                },
            ];

            let report = build_report(&fixtures_dir, &matches, &[]).unwrap();
            assert_eq!(
                report.leak_hits.len(),
                2,
                "unmapped and collision recordings must be leak-scanned too"
            );
            assert_eq!(report.exit_code(), 2);
        }

        #[test]
        fn build_report_never_writes_to_fixtures_dir() {
            let dir = tempdir().unwrap();
            let fixtures_dir = dir.path().join("fixtures");
            std::fs::create_dir_all(fixtures_dir.join("core")).unwrap();
            let fixture_path = fixtures_dir.join("core/billing_get.json");
            std::fs::write(&fixture_path, r#"{"Id":1}"#).unwrap();
            let before = std::fs::read(&fixture_path).unwrap();
            let before_mtime = std::fs::metadata(&fixture_path)
                .unwrap()
                .modified()
                .unwrap();

            let recorded_dir = dir.path().join("recorded");
            std::fs::create_dir_all(&recorded_dir).unwrap();
            let rec_path = recorded_dir.join("GET_billing.json");
            std::fs::write(&rec_path, r#"{"Id":1,"Extra":true}"#).unwrap();

            let matches = vec![RecordingMatch::Mapped {
                fixture_rel: "core/billing_get.json".to_string(),
                recording_rel: "core/GET_billing.json".to_string(),
                recording_abs: rec_path,
            }];

            let _report = build_report(&fixtures_dir, &matches, &[]).unwrap();

            let after = std::fs::read(&fixture_path).unwrap();
            let after_mtime = std::fs::metadata(&fixture_path)
                .unwrap()
                .modified()
                .unwrap();
            assert_eq!(before, after, "fixture bytes must be untouched");
            assert_eq!(before_mtime, after_mtime, "fixture mtime must be untouched");
        }
    }
}

// ---------------------------------------------------------------------------
// Tests for collect_recordings (top-level — exercises the encode/decode pair)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod collect_recordings_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn decodes_underscores_to_slashes_in_path() {
        // Recorder writes "GET_dnszone_50001.json" for path /dnszone/50001.
        // collect_recordings must decode underscores back to slashes so that
        // normalise_segments can split on '/' and produce ["dnszone","50001"].
        let dir = tempdir().unwrap();
        let domain_dir = dir.path().join("core");
        std::fs::create_dir_all(&domain_dir).unwrap();
        std::fs::write(domain_dir.join("GET_dnszone_50001.json"), b"{}").unwrap();

        let recordings = collect_recordings(dir.path()).expect("collect_recordings failed");
        assert_eq!(recordings.len(), 1);
        assert_eq!(recordings[0].method, "GET");
        assert_eq!(
            recordings[0].path, "/dnszone/50001",
            "path should decode underscores to slashes"
        );
    }

    #[test]
    fn root_sentinel_decodes_to_slash() {
        let dir = tempdir().unwrap();
        let domain_dir = dir.path().join("core");
        std::fs::create_dir_all(&domain_dir).unwrap();
        std::fs::write(domain_dir.join("GET_root.json"), b"{}").unwrap();

        let recordings = collect_recordings(dir.path()).expect("collect_recordings failed");
        assert_eq!(recordings.len(), 1);
        assert_eq!(recordings[0].path, "/");
    }
}
