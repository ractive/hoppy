//! Global `--dry-run` interception, shared by every domain client.
//!
//! Each client builds a full [`reqwest::Request`] before executing it; the
//! interception point is that shared moment, right after
//! `RequestBuilder::build()` and right before `http.execute()`. Read-only
//! requests (GET/HEAD/…) always proceed — see [`is_mutating`] — so a
//! composite command's preflight reads (resolving a storage-zone password,
//! a stream library key, linked pull zones, …) still run under `--dry-run`;
//! only the first mutating request in a composite is blocked.
//!
//! [`is_mutating`]: crate::recording::debug::is_mutating

use crate::recording::debug::{format_debug_body, is_mutating};

/// Error returned in place of actually sending a mutating request under
/// `--dry-run`.
///
/// Carries everything needed to render a preview: the method, the full URL
/// (including query string), and the (already redacted-per-`reveal`) body.
/// `main.rs` detects this by walking the returned `anyhow::Error` chain —
/// `err.chain().find_map(|e| e.downcast_ref::<DryRunSkipped>())` — which
/// survives `.context(...)` wrapping added by callers between the client and
/// `main`.
#[derive(Debug)]
pub struct DryRunSkipped {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
}

impl std::fmt::Display for DryRunSkipped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dry-run: would send {} {}", self.method, self.url)
    }
}

impl std::error::Error for DryRunSkipped {}

/// Block a built, mutating request when `dry_run` is set.
///
/// No-op (`Ok(())`) when `dry_run` is false or the request's method is not
/// mutating (per [`is_mutating`]) — read-only requests always execute.
///
/// The body is captured from the buffered request bytes when available
/// (redacted unless `reveal`), or rendered as `<streaming body>` for
/// chunked/streamed bodies (e.g. file uploads) whose bytes aren't available
/// up front.
pub fn check_dry_run(
    request: &reqwest::Request,
    dry_run: bool,
    reveal: bool,
) -> Result<(), DryRunSkipped> {
    if !dry_run || !is_mutating(request.method()) {
        return Ok(());
    }
    let body = match request.body().and_then(|b| b.as_bytes()) {
        Some(bytes) => Some(format_debug_body(bytes, reveal)),
        None if request.body().is_some() => Some("<streaming body>".to_owned()),
        None => None,
    };
    Err(DryRunSkipped {
        method: request.method().to_string(),
        url: request.url().to_string(),
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(method: reqwest::Method, url: &str, body: Option<&'static str>) -> reqwest::Request {
        let client = reqwest::Client::new();
        let mut rb = client.request(method, url);
        if let Some(b) = body {
            rb = rb.body(b);
        }
        rb.build().expect("valid request")
    }

    #[test]
    fn get_never_blocked() {
        let req = build(reqwest::Method::GET, "https://example.com/x", None);
        assert!(check_dry_run(&req, true, false).is_ok());
    }

    #[test]
    fn post_passes_through_when_dry_run_disabled() {
        let req = build(reqwest::Method::POST, "https://example.com/x", Some("{}"));
        assert!(check_dry_run(&req, false, false).is_ok());
    }

    #[test]
    fn post_blocked_under_dry_run_carries_method_url_body() {
        let req = build(
            reqwest::Method::POST,
            "https://example.com/x?y=1",
            Some(r#"{"a":1}"#),
        );
        let err = check_dry_run(&req, true, true).expect_err("must block");
        assert_eq!(err.method, "POST");
        assert_eq!(err.url, "https://example.com/x?y=1");
        assert!(err.body.expect("body present").contains("\"a\""));
    }

    #[test]
    fn delete_blocked_under_dry_run() {
        let req = build(reqwest::Method::DELETE, "https://example.com/x/1", None);
        assert!(check_dry_run(&req, true, false).is_err());
    }

    #[test]
    fn display_mentions_method_and_url() {
        let req = build(reqwest::Method::PATCH, "https://example.com/x", None);
        let err = check_dry_run(&req, true, false).expect_err("must block");
        let msg = err.to_string();
        assert!(msg.contains("PATCH"));
        assert!(msg.contains("https://example.com/x"));
    }
}
