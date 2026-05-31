---
title: Iter-44 — pull-zone security/compliance toggles
type: iteration
date: 2026-05-31
tags:
  - iteration
  - pull-zone
  - security
  - openapi-coverage
status: completed
branch: iter-44/pull-zone-security-compliance
---

# Iter-44 — pull-zone security/compliance toggles

## Why

Iter-43 (gap analysis) + iter-43b (dedup) confirmed pull-zone has
~152 unique missing properties between `PullZone`/`UpdatePullZone`
and the spec. The **security / compliance** bucket is the only one
that *blocks* PCI/SOC2 use cases (no way today to disable TLS 1.0/1.1,
verify origin SSL, or configure AWS SigV4 origin signing from
hoppy). Small, all booleans or short strings, ships as one surgical
PR. See [[../research/spec-coverage/pull-zone-buckets]] for the full
bucket categorisation.

## Scope

### 1. Add security/compliance fields to `PullZone` (read shape)

Source: `PullZoneModel` properties (specs/core-platform.json).

Add the following to `PullZone` in `crates/bunny-net-api/src/core/types.rs`,
matching the spec types and using `#[serde(default)]` for absent fields:

- [x] `enable_tls1: bool` (`EnableTLS1`)
- [x] `enable_tls1_1: bool` (`EnableTLS1_1`)
- [x] `enable_auto_ssl: bool` (`EnableAutoSSL`)
- [x] `disable_lets_encrypt: bool` (`DisableLetsEncrypt`)
- [x] `verify_origin_ssl: bool` (`VerifyOriginSSL`)
- [x] `enable_access_control_origin_header: bool` (`EnableAccessControlOriginHeader`)
- [x] `access_control_origin_header_extensions: Vec<String>` (`AccessControlOriginHeaderExtensions`)
- [x] `zone_security_include_hash_remote_ip: bool` (`ZoneSecurityIncludeHashRemoteIP`)
- [x] `aws_signing_enabled: bool` (`AWSSigningEnabled`)
- [x] `aws_signing_key: Option<String>` (`AWSSigningKey`) — write-only secret; `#[serde(skip_serializing)]` on read shape if API echoes it
- [x] `aws_signing_secret: Option<String>` (`AWSSigningSecret`) — write-only secret
- [x] `aws_signing_region_name: Option<String>` (`AWSSigningRegionName`)
- [x] `logging_ip_anonymization_enabled: bool` (`LoggingIPAnonymizationEnabled`)
- [x] `log_anonymization_type: Option<LogAnonymizationType>` (`LogAnonymizationType`) — new enum; check spec for variant list

### 2. Mirror onto `UpdatePullZone` (update payload)

Same 14 fields, all as `Option<T>` (sparse update semantics, matching
existing `UpdatePullZone` pattern).

- [x] Add all 14 fields to `UpdatePullZone`.
- [x] Confirm serde rename casing matches the API (PascalCase via existing
      crate-wide `rename_all = "PascalCase"` derive — verify by spot-checking
      one field with `curl` against the live API).

### 3. Add CLI flags to `hoppy pull-zone update`

In `crates/hoppy-cli/src/cli.rs` (around line 412, near
`monthly_bandwidth_limit`):

- [x] One `Option<bool>` clap arg per boolean toggle, kebab-cased.
- [x] `--access-control-origin-header-extensions` accepts comma-separated
      values (use the existing `Vec<String>` pattern from `blocked_ips` /
      `allowed_referrers` if one exists, otherwise `value_delimiter = ','`).
- [x] `--aws-signing-key` / `--aws-signing-secret`: mark help text with a
      warning that values appear in shell history; recommend `--from-env`
      or `--from-file` if those patterns exist elsewhere in the CLI.
- [x] `--log-anonymization-type` as an enum-valued arg.
- [x] Every new arg has `help = "..."` (the iter-41 §3 lesson — don't ship
      undocumented flags).

### 4. Tests + snapshots

- [x] Add unit tests for the new enum's serde round-trip.
- [x] `cargo test --workspace --quiet` clean.
- [x] Refresh e2e snapshots for `hoppy pull-zone update --help`.
- [x] Add one integration test that sends a sparse update with two new
      fields and verifies the wire payload (use the existing fixture
      machinery — see `tests/e2e/` in `bunny-net-api`).

## Out of scope

- Firewall, vary-headers, caching, origin/routing buckets — those are
  iter-45 / iter-46 / iter-47.
- Reading the cert provisioning state (separate `EdgeCertificates`
  endpoint — not in `PullZoneModel`).
- Removing the AWS signing secrets from `get`/`list` output if the
  API redacts them server-side; only suppress in our struct if the
  field comes back populated.

## Acceptance Criteria

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] `hoppy pull-zone update --help` lists all 14 new flags with help text.
- [x] Integration test verifies a sparse `UpdatePullZone` with
      `enable_tls1: Some(false)` serialises to `{"EnableTLS1": false}` only
      (no other fields in the payload).
- [x] `cargo run -p xtask -- check-iteration-ready --plan
      hoppy-knowledgebase/iterations/iteration-44-pull-zone-security-compliance.md
      --base origin/main` exits 0.

## Related

- [[../research/spec-coverage/pull-zone-buckets]] — bucket source
- [[iteration-43-openapi-gap-analysis]] — motivating audit
- [[../backlog/pull-zone-update-toggle-coverage-gap]] — original
  dogfooding finding
