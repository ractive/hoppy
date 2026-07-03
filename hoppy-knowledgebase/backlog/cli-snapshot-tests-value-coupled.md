---
title: CLI e2e snapshot tests are value-coupled to fixture values
date: 2026-05-14
type: backlog
status: completed
priority: medium
origin: iter-36
---

# CLI e2e snapshot tests are value-coupled to fixture values

## Problem

Several tests in `crates/hoppy-cli/tests/e2e/` use `insta::assert_snapshot!` on full command
output, or `assert!(stdout.contains("150000"))` style checks. These embed fixture values and
break whenever a fixture refresh changes the underlying data.

Affected tests (as of iter-36 drift simulation):

- `cli_auth::auth_check_table` — snapshot includes `$42.5000` balance from `billing_get.json`
- `cli_auth::auth_check_json` — snapshot includes balance and charges
- `cli_dns::dns_zone_list_json` — snapshot includes zone IDs from `dnszone_list_paginated.json`
- `cli_dns::dns_zone_list_table` — same
- `cli_pull_zone::pull_zone_create_json` — snapshot includes ID from `pullzone_get.json`
- `cli_pull_zone::pull_zone_get_json` — snapshot includes ID and name
- `cli_pull_zone::pull_zone_get_table` — same
- `cli_statistics::account_statistics_json` — `assert_eq!(json["TotalBandwidthUsed"], 5368709120_i64)`
- `cli_statistics::account_statistics_table` — `assert!(stdout.contains("150000"))`

## Fix

Replace value-coupled assertions with structural checks:

- For JSON output: parse and check key presence + types (`json["TotalBandwidthUsed"].is_number()`)
- For table output: check column headers are present, not specific cell values
- For `insta` snapshots: either replace with structural asserts, or scope the snapshot to
  the structural parts (JSON keys, table headers) rather than full output

## Related

- [[fixture-tests-assert-on-hardcoded-values]] — the original diagnosis
- [[iterations/iteration-36-shape-asserts]] — fixed the 4 `bunny-net-api` test files but
  left CLI tests as out-of-scope
- [[dogfooding/dogfooding-playbook]] — "Shape-first asserts in wiremock tests" section
