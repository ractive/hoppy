//! fixture-refresh — map recorded API responses to hand-authored wiremock fixtures.
//!
//! Usage:
//!   fixture-refresh --recorded <DIR> [--fixtures <DIR>] [--apply]
//!
//! Workflow:
//! 1. Run the live test suite with `HOPPY_RECORD_DIR=<scratch>` to capture fresh
//!    responses with auto-derived filenames like `core/GET_dnszone_50001.json`.
//! 2. Run `fixture-refresh --recorded <scratch>` (dry-run) to preview what would change.
//! 3. Add `--apply` to overwrite the hand-authored descriptive fixtures that drifted.

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
    about = "Map recorded API responses to hand-authored wiremock fixtures and apply drift"
)]
struct Cli {
    /// Directory of recording outputs (e.g. fixtures-recorded/ produced by HOPPY_RECORD_DIR=...)
    #[arg(long)]
    recorded: PathBuf,

    /// Root of the descriptive-name fixture tree to refresh (default: fixtures/)
    #[arg(long, default_value = "fixtures")]
    fixtures: PathBuf,

    /// Actually overwrite drifted fixtures (default is dry-run)
    #[arg(long)]
    apply: bool,
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

    // §3 — Diff and optionally apply.
    let mut drifted = 0usize;
    let mut identical = 0usize;
    let mut collisions = 0usize;
    let mut unmapped = 0usize;

    for m in &matches {
        match m {
            RecordingMatch::Mapped {
                fixture_rel,
                recording_abs,
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
                    if cli.apply {
                        std::fs::write(&fixture_abs, &rec_bytes)
                            .with_context(|| format!("writing {}", fixture_abs.display()))?;
                        println!("applied: {} (Δ {} bytes)", fixture_rel, delta);
                    } else {
                        println!("drift:   {} (Δ {} bytes)", fixture_rel, delta);
                    }
                } else {
                    // Fixture doesn't exist on disk — count as drift (new file)
                    drifted += 1;
                    if cli.apply {
                        if let Some(parent) = fixture_abs.parent() {
                            std::fs::create_dir_all(parent)
                                .with_context(|| format!("creating dir {}", parent.display()))?;
                        }
                        std::fs::write(&fixture_abs, &rec_bytes)
                            .with_context(|| format!("writing {}", fixture_abs.display()))?;
                        println!("applied (new): {} ({} bytes)", fixture_rel, rec_bytes.len());
                    } else {
                        println!("drift (new): {} ({} bytes)", fixture_rel, rec_bytes.len());
                    }
                }
            }
            RecordingMatch::Collision {
                recording_rel,
                candidates,
            } => {
                collisions += 1;
                println!("collision: {} → [{}]", recording_rel, candidates.join(", "));
            }
            RecordingMatch::Unmapped { recording_rel } => {
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
// Match result types (shared between matcher and main)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RecordingMatch {
    /// Recording matched exactly one descriptive fixture.
    Mapped {
        fixture_rel: String,
        recording_abs: PathBuf,
    },
    /// Recording matched multiple descriptive fixtures (ambiguous — skip).
    Collision {
        recording_rel: String,
        candidates: Vec<String>,
    },
    /// No descriptive fixture maps to this recording.
    Unmapped { recording_rel: String },
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
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
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
        let stem = filename.trim_end_matches(".json");
        let Some(underscore_pos) = stem.find('_') else {
            continue;
        };
        let method = stem[..underscore_pos].to_uppercase();
        let segments_part = &stem[underscore_pos + 1..];
        let path_str = if segments_part == "root" {
            "/".to_string()
        } else {
            format!("/{}", segments_part)
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
        for entry in WalkDir::new(&crates_dir)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target" && name != ".git"
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
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
                    },
                    1 => RecordingMatch::Mapped {
                        fixture_rel: deduped.into_iter().next().unwrap(),
                        recording_abs: rec.abs.clone(),
                    },
                    _ => RecordingMatch::Collision {
                        recording_rel: rec.rel.clone(),
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
                // Numeric recording segment → matches any fixture segment
                if is_numeric(r) {
                    return true;
                }
                // Underscore-to-slash ambiguity: the recording encodes path slashes
                // as underscores, so a recording path like /dnszone_50001 might
                // correspond to fixture path /dnszone/50001. However the recording
                // module does it correctly: it builds the filename by replacing '/'
                // with '_', so we reconstruct the path by looking at segments only.
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
    }
}
