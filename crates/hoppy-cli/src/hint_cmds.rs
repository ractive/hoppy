//! Command lines suggested to the user in `tip:` hints and error messages.
//!
//! Keeping them in one place lets the tests below run every suggestion through
//! clap, so a hint can never drift from the real CLI surface (as happened when
//! iter-86 made the id positional but the hints kept saying `--id`).
//!
//! Commands that still need `--id` (see `backlog/id-arg-remaining-commands`)
//! are listed explicitly and covered by a test that fails once they accept the
//! positional form.

pub fn pull_zone_get(id: i64) -> String {
    format!("hoppy pull-zone get {id}")
}

pub fn pull_zone_statistics(id: i64) -> String {
    format!("hoppy pull-zone statistics {id}")
}

pub fn pull_zone_hostname_add(id: i64) -> String {
    format!("hoppy pull-zone hostname add {id} --hostname <fqdn>")
}

pub fn pull_zone_edge_rule_add(id: i64) -> String {
    format!("hoppy pull-zone edge-rule add {id} ...")
}

pub fn pull_zone_delete(id: impl std::fmt::Display) -> String {
    format!("hoppy pull-zone delete {id} --yes")
}

pub fn pull_zone_update_blocked_countries(id: impl std::fmt::Display) -> String {
    format!("hoppy pull-zone update {id} --blocked-countries <CODE>")
}

pub fn billing_payment_request_pdf(id: impl std::fmt::Display) -> String {
    format!("hoppy billing payment-request-pdf {id} --output invoice.pdf")
}

pub fn storage_zone_get(id: i64) -> String {
    format!("hoppy storage-zone get {id}")
}

pub fn dns_zone_get(id: i64) -> String {
    format!("hoppy dns zone get {id}")
}

/// Still `--id` only: `--id` and `--domain` are alternatives.
pub fn dns_zone_scan_results(id: i64) -> String {
    format!("hoppy dns zone scan results --id {id}")
}

pub fn stream_video_list(library_id: i64) -> String {
    format!("hoppy stream video list --library-id {library_id}")
}

/// Domain alternative to [`dns_zone_scan_results`].
pub fn dns_zone_scan_results_domain(domain: &str) -> String {
    format!("hoppy dns zone scan results --domain {domain}")
}

pub fn stream_library_get(id: i64) -> String {
    format!("hoppy stream library get {id}")
}

pub fn container_app_get(id: &str) -> String {
    format!("hoppy container app get {id}")
}

/// Still `--id` only: the shield commands use `--id`/`--shield-zone-id`.
pub fn shield_zone_get(id: i64) -> String {
    format!("hoppy shield zone get --id {id}")
}

/// Drop the leading `hoppy ` for the `jq` drill-down hint.
pub fn without_binary(cmd: &str) -> &str {
    cmd.strip_prefix("hoppy ").unwrap_or(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use clap::Parser;
    use clap::error::ErrorKind;

    /// Hints that take a positional id.
    fn positional_hints() -> Vec<String> {
        vec![
            pull_zone_get(5871708),
            pull_zone_statistics(5871708),
            pull_zone_hostname_add(5871708),
            pull_zone_edge_rule_add(5871708),
            pull_zone_delete(5871708),
            storage_zone_get(1520401),
            dns_zone_get(50001),
            stream_library_get(77),
            stream_video_list(77),
            dns_zone_scan_results_domain("example.com"),
            container_app_get("app-1"),
            pull_zone_update_blocked_countries(5871708),
            billing_payment_request_pdf(42),
        ]
    }

    /// Hints that must keep `--id`.
    fn flag_hints() -> Vec<String> {
        vec![shield_zone_get(12), dns_zone_scan_results(50001)]
    }

    /// `true` when clap accepts the command (placeholders such as `...` leave
    /// required arguments missing, which is fine: the subcommand and the id
    /// were still recognised).
    fn parses(hint: &str) -> bool {
        let hint = hint.strip_suffix(" ...").unwrap_or(hint);
        match Cli::try_parse_from(hint.split_whitespace()) {
            Ok(_) => true,
            Err(e) => e.kind() == ErrorKind::MissingRequiredArgument,
        }
    }

    #[test]
    fn every_hint_parses_with_clap() {
        for hint in positional_hints().iter().chain(flag_hints().iter()) {
            assert!(parses(hint), "hint does not parse: {hint}");
        }
    }

    #[test]
    fn positional_hints_do_not_use_id_flag() {
        for hint in positional_hints() {
            assert!(!hint.contains("--id"), "{hint}");
        }
    }

    #[test]
    fn flag_hints_reject_positional_form() {
        // Fails once the command gains a positional id: switch the hint then.
        for hint in flag_hints() {
            let positional = hint.replace("--id ", "");
            assert!(!parses(&positional), "now positional, update hint: {hint}");
        }
    }
}
