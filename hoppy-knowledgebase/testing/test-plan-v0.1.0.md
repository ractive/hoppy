---
title: "Test Plan: hoppy v0.1.0 Pre-Release"
type: test-plan
status: draft
created: 2026-03-19
scope: full CLI + API client coverage
---

# Test Plan: hoppy v0.1.0 Pre-Release

## Overview

Comprehensive test plan covering all CLI commands, flags, API client methods,
output formatting, error handling, and help text accuracy before the first release.

**Scope:** 10 top-level commands, 70+ subcommands, 200+ flags, 6 API client crates

---

## Part 0: Implementation Gap Analysis

Items found in the API clients or Bunny.net docs that are **not** exposed as CLI commands.
These should be evaluated for inclusion in v0.1.0 or documented as known limitations.

| # | Feature | API Client | CLI | Notes |
|---|---------|-----------|-----|-------|
| 1 | `stream video update` | `bunny-api-stream::update_video` | **Done** | Added in iter-9 |
| 2 | `stream video fetch` (from URL) | `bunny-api-stream::fetch_video` | **Done** | Added in iter-9 |
| 3 | Stream collection CRUD | `bunny-api-stream` has full CRUD | **Done** | Added in iter-9 |
| 4 | `script rotate-deployment-key` | `bunny-api-compute::rotate_deployment_key` | **Done** | Added in iter-9 |
| 5 | `list_variables` (compute) | Missing from client | N/A | CLI works around it via `get_script` response |
| 6 | DNS zone export/import (BIND) | Not implemented | Missing | Documented in Bunny.net API |
| 7 | Core API statistics | Not implemented | Missing | GET `/statistics` for traffic/bandwidth |
| 8 | Core API region listing | Not implemented | Missing | GET `/region` for available regions |
| 9 | `pull-zone create/update` | Limited fields | Limited | Many pull zone settings not exposed |
| 10 | Stream advanced features | Not implemented | Missing | Captions, transcribe, re-encode, heatmap, resolutions |

**Status:** Items 1-4 were implemented in this iteration. Items 5-10 are deferred.

---

## Part 1: Help Text Accuracy Tests

Verify that `--help` output for every command is accurate, complete, and consistent.

### 1.1 Top-Level Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.1.1 | Root help lists all commands | `hoppy --help` | All 10 commands listed (auth, pull-zone, storage-zone, storage, dns, stream, shield, script, container, completions) |
| 1.1.2 | Root help shows global flags | `hoppy --help` | Shows --format, --debug, --quiet, --yes/-y |
| 1.1.3 | Root help shows version | `hoppy --version` | Shows version string |

### 1.2 Auth Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.2.1 | Auth subcommands | `hoppy auth --help` | Lists `check` subcommand |
| 1.2.2 | Auth check help | `hoppy auth check --help` | No extra flags beyond global |

### 1.3 Pull Zone Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.3.1 | Pull zone subcommands | `hoppy pull-zone --help` | Lists: list, get, create, update, delete, purge |
| 1.3.2 | Pull zone list flags | `hoppy pull-zone list --help` | --search, --page, --per-page |
| 1.3.3 | Pull zone get flags | `hoppy pull-zone get --help` | --id (required) |
| 1.3.4 | Pull zone create flags | `hoppy pull-zone create --help` | --name, --origin-url (both required) |
| 1.3.5 | Pull zone update flags | `hoppy pull-zone update --help` | --id (required), --origin-url, --monthly-bandwidth-limit, --cache-expiration-time, --zone-security-enabled, --enable-geo-zone-us/eu/asia/sa/af |
| 1.3.6 | Pull zone delete flags | `hoppy pull-zone delete --help` | --id (required) |
| 1.3.7 | Pull zone purge flags | `hoppy pull-zone purge --help` | --id (required), --cache-tag (optional) |

### 1.4 Storage Zone Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.4.1 | Storage zone subcommands | `hoppy storage-zone --help` | Lists: list, get, create, update, delete |
| 1.4.2 | Storage zone list flags | `hoppy storage-zone list --help` | --search, --page, --per-page |
| 1.4.3 | Storage zone get flags | `hoppy storage-zone get --help` | --id (required) |
| 1.4.4 | Storage zone create flags | `hoppy storage-zone create --help` | --name, --region (required), --replication-regions, --zone-tier |
| 1.4.5 | Storage zone update flags | `hoppy storage-zone update --help` | --id (required), --rewrite-404-to-200, --custom-404-file-path, --origin-url |
| 1.4.6 | Storage zone delete flags | `hoppy storage-zone delete --help` | --id (required) |

### 1.5 Storage (File Ops) Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.5.1 | Storage subcommands | `hoppy storage --help` | Lists: upload, download, ls, rm |
| 1.5.2 | Storage upload flags | `hoppy storage upload --help` | --zone, --remote-path, --file (required), --region (optional) |
| 1.5.3 | Storage download flags | `hoppy storage download --help` | --zone, --remote-path (required), --output, --region |
| 1.5.4 | Storage ls flags | `hoppy storage ls --help` | --zone (required), --path, --region |
| 1.5.5 | Storage rm flags | `hoppy storage rm --help` | --zone, --remote-path (required), --region |

### 1.6 DNS Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.6.1 | DNS subcommands | `hoppy dns --help` | Lists: zone, record |
| 1.6.2 | DNS zone subcommands | `hoppy dns zone --help` | Lists: list, get, create, update, delete |
| 1.6.3 | DNS zone list flags | `hoppy dns zone list --help` | --search, --page, --per-page |
| 1.6.4 | DNS zone get flags | `hoppy dns zone get --help` | --id (required) |
| 1.6.5 | DNS zone create flags | `hoppy dns zone create --help` | --domain (required) |
| 1.6.6 | DNS zone update flags | `hoppy dns zone update --help` | --id (required), --custom-nameservers-enabled, --nameserver1, --nameserver2, --soa-email, --logging-enabled, --logging-ip-anonymization-enabled |
| 1.6.7 | DNS zone delete flags | `hoppy dns zone delete --help` | --id (required) |
| 1.6.8 | DNS record subcommands | `hoppy dns record --help` | Lists: list, add, update, delete |
| 1.6.9 | DNS record list flags | `hoppy dns record list --help` | --zone-id (required) |
| 1.6.10 | DNS record add flags | `hoppy dns record add --help` | --zone-id, --type, --value (required), --name, --ttl, --priority, --weight, --port, --flags, --tag, --comment |
| 1.6.11 | DNS record update flags | `hoppy dns record update --help` | --zone-id, --record-id, --type, --value (required), --name, --ttl, --priority, --weight, --comment |
| 1.6.12 | DNS record delete flags | `hoppy dns record delete --help` | --zone-id, --record-id (required) |

### 1.7 Stream Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.7.1 | Stream subcommands | `hoppy stream --help` | Lists: library, video, collection |
| 1.7.2 | Stream library subcommands | `hoppy stream library --help` | Lists: list, get, create, update, delete |
| 1.7.3 | Stream library list flags | `hoppy stream library list --help` | --search, --page, --per-page |
| 1.7.4 | Stream library get flags | `hoppy stream library get --help` | --id (required) |
| 1.7.5 | Stream library create flags | `hoppy stream library create --help` | --name (required) |
| 1.7.6 | Stream library update flags | `hoppy stream library update --help` | --id (required), --name, --allow-direct-play, --enable-mp4-fallback, --has-watermark |
| 1.7.7 | Stream library delete flags | `hoppy stream library delete --help` | --id (required) |
| 1.7.8 | Stream video subcommands | `hoppy stream video --help` | Lists: list, get, upload, update, fetch, delete |
| 1.7.9 | Stream video list flags | `hoppy stream video list --help` | --library-id (required), --page, --items-per-page, --search, --collection, --order-by |
| 1.7.10 | Stream video get flags | `hoppy stream video get --help` | --library-id, --video-id (required) |
| 1.7.11 | Stream video upload flags | `hoppy stream video upload --help` | --library-id, --file (required), --title, --collection-id |
| 1.7.12 | Stream video update flags | `hoppy stream video update --help` | --library-id, --video-id (required), --title, --collection-id |
| 1.7.13 | Stream video fetch flags | `hoppy stream video fetch --help` | --library-id, --url (required), --title |
| 1.7.14 | Stream video delete flags | `hoppy stream video delete --help` | --library-id, --video-id (required) |
| 1.7.15 | Stream collection subcommands | `hoppy stream collection --help` | Lists: list, get, create, update, delete |
| 1.7.16 | Stream collection list flags | `hoppy stream collection list --help` | --library-id (required), --page, --items-per-page, --search, --order-by |
| 1.7.17 | Stream collection get flags | `hoppy stream collection get --help` | --library-id, --collection-id (required) |
| 1.7.18 | Stream collection create flags | `hoppy stream collection create --help` | --library-id, --name (required) |
| 1.7.19 | Stream collection update flags | `hoppy stream collection update --help` | --library-id, --collection-id (required), --name |
| 1.7.20 | Stream collection delete flags | `hoppy stream collection delete --help` | --library-id, --collection-id (required) |

### 1.8 Shield Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.8.1 | Shield subcommands | `hoppy shield --help` | Lists: zone, waf, rate-limit, access-list, bot-detection |
| 1.8.2 | Shield zone subcommands | `hoppy shield zone --help` | Lists: list, get, get-by-pullzone, create, update |
| 1.8.3 | Shield zone get flags | `hoppy shield zone get --help` | --shield-zone-id (required) |
| 1.8.4 | Shield zone get-by-pullzone | `hoppy shield zone get-by-pullzone --help` | --pull-zone-id (required) |
| 1.8.5 | Shield zone create flags | `hoppy shield zone create --help` | --pull-zone-id (required) |
| 1.8.6 | Shield zone update flags | `hoppy shield zone update --help` | --shield-zone-id (required), --waf-enabled, --waf-execution-mode, --ddos-sensitivity, --ddos-execution-mode, --ddos-challenge-window, --learning-mode |
| 1.8.7 | Shield waf subcommands | `hoppy shield waf --help` | Lists: profiles, list-rules, get-rule, add-rule, update-rule, delete-rule |
| 1.8.8 | Shield waf add-rule flags | `hoppy shield waf add-rule --help` | --shield-zone-id, --action-type, --operator-type, --severity-type (required), --name, --value |
| 1.8.9 | Shield waf update-rule flags | `hoppy shield waf update-rule --help` | --id (required), --name |
| 1.8.10 | Shield waf delete-rule flags | `hoppy shield waf delete-rule --help` | --id (required) |
| 1.8.11 | Shield rate-limit subcommands | `hoppy shield rate-limit --help` | Lists: list, get, create, update, delete |
| 1.8.12 | Shield rate-limit create flags | `hoppy shield rate-limit create --help` | --shield-zone-id, --action-type, --operator-type, --severity-type, --request-count, --counter-key-type, --timeframe, --block-time (required), --name, --value |
| 1.8.13 | Shield rate-limit update flags | `hoppy shield rate-limit update --help` | --id (required), --name |
| 1.8.14 | Shield rate-limit delete flags | `hoppy shield rate-limit delete --help` | --id (required) |
| 1.8.15 | Shield access-list subcommands | `hoppy shield access-list --help` | Lists: list, get, create, update, delete, update-config |
| 1.8.16 | Shield access-list create flags | `hoppy shield access-list create --help` | --shield-zone-id, --name, --type, --content (required) |
| 1.8.17 | Shield access-list update flags | `hoppy shield access-list update --help` | --shield-zone-id, --id (required), --name, --content |
| 1.8.18 | Shield access-list update-config | `hoppy shield access-list update-config --help` | --shield-zone-id, --configuration-id (required), --is-enabled, --action |
| 1.8.19 | Shield access-list delete flags | `hoppy shield access-list delete --help` | --shield-zone-id, --id (required) |
| 1.8.20 | Shield bot-detection subcommands | `hoppy shield bot-detection --help` | Lists: get, update |
| 1.8.21 | Shield bot-detection get flags | `hoppy shield bot-detection get --help` | --shield-zone-id (required) |
| 1.8.22 | Shield bot-detection update flags | `hoppy shield bot-detection update --help` | --shield-zone-id (required), --execution-mode, --request-integrity-sensitivity, --ip-address-sensitivity, --fingerprint-sensitivity, --fingerprint-aggression, --fingerprint-complex-enabled |

### 1.9 Script Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.9.1 | Script subcommands | `hoppy script --help` | Lists: list, get, create, update, delete, publish, statistics, code, release, variable, secret, rotate-deployment-key |
| 1.9.2 | Script list flags | `hoppy script list --help` | --search, --page, --per-page |
| 1.9.3 | Script get flags | `hoppy script get --help` | --id (required) |
| 1.9.4 | Script create flags | `hoppy script create --help` | --name, --script-type, --create-linked-pull-zone (required), --code, --linked-pull-zone-name |
| 1.9.5 | Script update flags | `hoppy script update --help` | --id (required), --name, --script-type |
| 1.9.6 | Script delete flags | `hoppy script delete --help` | --id, --delete-linked-pull-zones (required) |
| 1.9.7 | Script publish flags | `hoppy script publish --help` | --id (required), --note |
| 1.9.8 | Script statistics flags | `hoppy script statistics --help` | --id (required), --date-from, --date-to, --hourly |
| 1.9.9 | Script code subcommands | `hoppy script code --help` | Lists: get, update |
| 1.9.10 | Script code get flags | `hoppy script code get --help` | --id (required) |
| 1.9.11 | Script code update flags | `hoppy script code update --help` | --id (required), --code, --file (mutually exclusive) |
| 1.9.12 | Script release subcommands | `hoppy script release --help` | Lists: list, get-active |
| 1.9.13 | Script release list flags | `hoppy script release list --help` | --id (required), --page, --per-page |
| 1.9.14 | Script release get-active flags | `hoppy script release get-active --help` | --id (required) |
| 1.9.15 | Script variable subcommands | `hoppy script variable --help` | Lists: list, add, update, delete, upsert |
| 1.9.16 | Script variable add flags | `hoppy script variable add --help` | --id, --name, --required (required), --default-value |
| 1.9.17 | Script variable update flags | `hoppy script variable update --help` | --id, --variable-id (required), --required, --default-value |
| 1.9.18 | Script variable delete flags | `hoppy script variable delete --help` | --id, --variable-id (required) |
| 1.9.19 | Script variable upsert flags | `hoppy script variable upsert --help` | --id, --name (required), --required, --default-value |
| 1.9.20 | Script secret subcommands | `hoppy script secret --help` | Lists: list, add, update, delete, upsert |
| 1.9.21 | Script secret add flags | `hoppy script secret add --help` | --id, --name, --value (required) |
| 1.9.22 | Script secret update flags | `hoppy script secret update --help` | --id, --secret-id, --value (required) |
| 1.9.23 | Script secret delete flags | `hoppy script secret delete --help` | --id, --secret-id (required) |
| 1.9.24 | Script secret upsert flags | `hoppy script secret upsert --help` | --id, --name, --value (required) |

### 1.10 Container Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.10.1 | Container subcommands | `hoppy container --help` | Lists: app, template, endpoint, volume, registry, region, node, pod, limits, log-forwarding |
| 1.10.2 | Container app subcommands | `hoppy container app --help` | Lists: list, get, create, update, deploy, undeploy, restart, delete, overview, statistics, autoscaling-get, autoscaling-update, region-settings-get, region-settings-update |
| 1.10.3 | Container app list flags | `hoppy container app list --help` | --cursor, --limit |
| 1.10.4 | Container app get flags | `hoppy container app get --help` | --id (required) |
| 1.10.5 | Container app create flags | `hoppy container app create --help` | --name, --runtime-type, --min, --max, --region (all required) |
| 1.10.6 | Container app update flags | `hoppy container app update --help` | --id (required), --name, --runtime-type, --min, --max |
| 1.10.7 | Container app deploy/undeploy/restart/delete | `hoppy container app deploy --help` | --id (required) |
| 1.10.8 | Container app overview flags | `hoppy container app overview --help` | --id (required) |
| 1.10.9 | Container app statistics flags | `hoppy container app statistics --help` | --id, --from (required), --to, --granularity |
| 1.10.10 | Container app autoscaling-get | `hoppy container app autoscaling-get --help` | --app-id (required) |
| 1.10.11 | Container app autoscaling-update | `hoppy container app autoscaling-update --help` | --app-id, --min, --max (required) |
| 1.10.12 | Container app region-settings-get | `hoppy container app region-settings-get --help` | --app-id (required) |
| 1.10.13 | Container app region-settings-update | `hoppy container app region-settings-update --help` | --app-id (required), --allowed-region, --required-region, --max-allowed-regions |
| 1.10.14 | Container template subcommands | `hoppy container template --help` | Lists: get, add, update, delete, env |
| 1.10.15 | Container template add flags | `hoppy container template add --help` | --app-id, --name, --image-name, --image-namespace, --image-tag, --registry-id (all required) |
| 1.10.16 | Container template update flags | `hoppy container template update --help` | --app-id, --container-id (required), --name, --image-tag, --image-name, --image-namespace, --registry-id |
| 1.10.17 | Container template env flags | `hoppy container template env --help` | --app-id, --container-id, --env (required) |
| 1.10.18 | Container endpoint subcommands | `hoppy container endpoint --help` | Lists: list, add, update, delete |
| 1.10.19 | Container endpoint add flags | `hoppy container endpoint add --help` | --app-id, --container-id, --name, --container-port (required), --exposed-port, --cdn, --anycast (mutually exclusive) |
| 1.10.20 | Container endpoint update flags | `hoppy container endpoint update --help` | --app-id, --endpoint-id, --name, --container-port (required), --exposed-port, --cdn, --anycast |
| 1.10.21 | Container volume subcommands | `hoppy container volume --help` | Lists: list, update, detach, delete, delete-instance |
| 1.10.22 | Container volume update flags | `hoppy container volume update --help` | --app-id, --volume-id (required), --name, --size |
| 1.10.23 | Container volume delete-instance | `hoppy container volume delete-instance --help` | --app-id, --volume-id, --instance-id (required) |
| 1.10.24 | Container registry subcommands | `hoppy container registry --help` | Lists: list, get, create, update, delete, image-tags, image-digest, config-suggestions, search-public |
| 1.10.25 | Container registry create flags | `hoppy container registry create --help` | --name (required), --registry-type, --username, --password |
| 1.10.26 | Container registry image-tags | `hoppy container registry image-tags --help` | --registry-id, --image-name, --image-namespace (required) |
| 1.10.27 | Container registry image-digest | `hoppy container registry image-digest --help` | --registry-id, --image-name, --image-namespace, --tag (required) |
| 1.10.28 | Container registry config-suggestions | `hoppy container registry config-suggestions --help` | --registry-id, --image-name, --image-namespace, --tag (required) |
| 1.10.29 | Container registry search-public | `hoppy container registry search-public --help` | --registry-id, --query (required), --size, --page |
| 1.10.30 | Container region subcommands | `hoppy container region --help` | Lists: list, optimal |
| 1.10.31 | Container region list flags | `hoppy container region list --help` | --cursor, --limit |
| 1.10.32 | Container node subcommands | `hoppy container node --help` | Lists: list |
| 1.10.33 | Container node list flags | `hoppy container node list --help` | --cursor, --limit |
| 1.10.34 | Container pod subcommands | `hoppy container pod --help` | Lists: recreate |
| 1.10.35 | Container pod recreate flags | `hoppy container pod recreate --help` | --app-id, --pod-id (required) |
| 1.10.36 | Container limits | `hoppy container limits --help` | No extra flags |
| 1.10.37 | Container log-forwarding subcommands | `hoppy container log-forwarding --help` | Lists: list, get, create, update, delete |
| 1.10.38 | Container log-forwarding create | `hoppy container log-forwarding create --help` | --app-id, --forwarding-type, --endpoint, --port, --format, --enabled (required), --token |
| 1.10.39 | Container log-forwarding update | `hoppy container log-forwarding update --help` | --app-id, --forwarding-type, --endpoint, --port, --format, --enabled (required), --token |
| 1.10.40 | Container log-forwarding delete | `hoppy container log-forwarding delete --help` | --app-id (required) |

### 1.11 Completions Help

| # | Test | Command | Verify |
|---|------|---------|--------|
| 1.11.1 | Completions help | `hoppy completions --help` | Shows SHELL positional arg, lists shells |

---

## Part 2: Global Flag Tests

| # | Test | How | Expected |
|---|------|-----|----------|
| 2.1 | `--format json` produces valid JSON | `hoppy pull-zone list --format json` | Valid JSON array output |
| 2.2 | `--format table` produces table | `hoppy pull-zone list --format table` | Table with headers and rows |
| 2.3 | `--format text` produces text | `hoppy pull-zone list --format text` | Plain text output |
| 2.4 | `--debug` shows HTTP requests | `hoppy auth check --debug` | stderr shows HTTP method, URL, headers |
| 2.5 | `--debug` redacts API key | `hoppy auth check --debug` | AccessKey header value is masked |
| 2.6 | `--quiet` suppresses output | `hoppy pull-zone delete --id 123 --quiet --yes` | No confirmation text, minimal output |
| 2.7 | `--yes` skips confirmation | `hoppy pull-zone delete --id 123 --yes` | No interactive prompt |
| 2.8 | Default format is table | `hoppy pull-zone list` (no --format) | Table output |
| 2.9 | Global flags work with all commands | Test --format json on every list/get command | All produce valid JSON |
| 2.10 | Unknown flag rejected | `hoppy --unknown-flag` | Error message, non-zero exit |

---

## Part 3: Command Functional Tests (Wiremock Integration)

These tests verify that CLI commands correctly call the API and format output.
All should use wiremock to mock HTTP responses.

### 3.1 Auth

| # | Test | Verify |
|---|------|--------|
| 3.1.1 | `auth check` with valid key | Displays billing info (balance, charges) |
| 3.1.2 | `auth check` with invalid key | Error message, non-zero exit |
| 3.1.3 | `auth check` with no key configured | Helpful error about missing API key |

### 3.2 Pull Zone

| # | Test | Verify |
|---|------|--------|
| 3.2.1 | `pull-zone list` | Table with zone ID, name, origin URL |
| 3.2.2 | `pull-zone list --search "test"` | Filtered results |
| 3.2.3 | `pull-zone list --page 2 --per-page 5` | Pagination params sent to API |
| 3.2.4 | `pull-zone list --format json` | Valid JSON array |
| 3.2.5 | `pull-zone get --id 123` | Single zone details |
| 3.2.6 | `pull-zone get --id 999` (not found) | Error message |
| 3.2.7 | `pull-zone create --name test --origin-url https://example.com` | Success, shows created zone |
| 3.2.8 | `pull-zone create` missing required flags | Clap error listing required flags |
| 3.2.9 | `pull-zone update --id 123 --origin-url https://new.com` | Success |
| 3.2.10 | `pull-zone update --id 123` with no optional flags | Sends empty/minimal update |
| 3.2.11 | `pull-zone update` all optional flags | Each field sent correctly |
| 3.2.12 | `pull-zone delete --id 123` | Confirmation prompt, then deletes |
| 3.2.13 | `pull-zone delete --id 123 --yes` | No prompt, deletes directly |
| 3.2.14 | `pull-zone purge --id 123` | Purges entire cache |
| 3.2.15 | `pull-zone purge --id 123 --cache-tag "css"` | Purges by tag |

### 3.3 Storage Zone

| # | Test | Verify |
|---|------|--------|
| 3.3.1 | `storage-zone list` | Table with zone details |
| 3.3.2 | `storage-zone list --search "media"` | Filtered results |
| 3.3.3 | `storage-zone list --page 1 --per-page 10` | Pagination |
| 3.3.4 | `storage-zone get --id 123` | Single zone details |
| 3.3.5 | `storage-zone get --id 999` (not found) | Error |
| 3.3.6 | `storage-zone create --name test --region DE` | Creates zone |
| 3.3.7 | `storage-zone create --name test --region DE --replication-regions NY,LA` | Creates with replication |
| 3.3.8 | `storage-zone create --name test --region DE --zone-tier 1` | Edge tier |
| 3.3.9 | `storage-zone update --id 123 --origin-url https://origin.com` | Updates |
| 3.3.10 | `storage-zone delete --id 123` | Confirmation + delete |
| 3.3.11 | `storage-zone delete --id 123 --yes` | Skip confirmation |

### 3.4 Storage (File Operations)

| # | Test | Verify |
|---|------|--------|
| 3.4.1 | `storage ls --zone myzone` | Lists root directory |
| 3.4.2 | `storage ls --zone myzone --path images` | Lists subdirectory |
| 3.4.3 | `storage ls --zone myzone --region la` | Uses LA region endpoint |
| 3.4.4 | `storage upload --zone myzone --remote-path img/a.jpg --file ./a.jpg` | Uploads file, correct Content-Type |
| 3.4.5 | `storage upload` with non-existent local file | Helpful error |
| 3.4.6 | `storage upload --region sg` | Uses SG region endpoint |
| 3.4.7 | `storage download --zone myzone --remote-path img/a.jpg` | Downloads to stdout |
| 3.4.8 | `storage download --output ./local.jpg` | Writes to file |
| 3.4.9 | `storage rm --zone myzone --remote-path img/a.jpg` | Deletes file |
| 3.4.10 | `storage rm` confirmation behavior | Prompts or respects --yes |

### 3.5 DNS

| # | Test | Verify |
|---|------|--------|
| 3.5.1 | `dns zone list` | Lists zones |
| 3.5.2 | `dns zone list --search "example.com"` | Filtered |
| 3.5.3 | `dns zone get --id 123` | Zone with records |
| 3.5.4 | `dns zone create --domain example.com` | Creates zone |
| 3.5.5 | `dns zone update --id 123 --logging-enabled true` | Updates |
| 3.5.6 | `dns zone update` all optional flags | All fields sent |
| 3.5.7 | `dns zone delete --id 123` | Confirmation + delete |
| 3.5.8 | `dns record list --zone-id 123` | Lists records for zone |
| 3.5.9 | `dns record add --zone-id 123 --type A --value 1.2.3.4` | Adds A record |
| 3.5.10 | `dns record add --zone-id 123 --type A --value 1.2.3.4 --name www` | Adds subdomain record |
| 3.5.11 | `dns record add --zone-id 123 --type MX --value mail.example.com --priority 10` | MX with priority |
| 3.5.12 | `dns record add --zone-id 123 --type SRV --value target.com --priority 10 --weight 5 --port 443` | SRV record |
| 3.5.13 | `dns record add --zone-id 123 --type CAA --value "letsencrypt.org" --flags 0 --tag issue` | CAA record |
| 3.5.14 | `dns record add` all record types | A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, Redirect, Flatten, PullZone, Script, SVCB, HTTPS, TLSA |
| 3.5.15 | `dns record update --zone-id 123 --record-id 456 --type A --value 5.6.7.8` | Updates record |
| 3.5.16 | `dns record delete --zone-id 123 --record-id 456` | Deletes record |

### 3.6 Stream

| # | Test | Verify |
|---|------|--------|
| 3.6.1 | `stream library list` | Lists libraries |
| 3.6.2 | `stream library list --search "videos"` | Filtered |
| 3.6.3 | `stream library get --id 123` | Library details |
| 3.6.4 | `stream library create --name "My Library"` | Creates |
| 3.6.5 | `stream library update --id 123 --name "New Name"` | Updates |
| 3.6.6 | `stream library update` all optional flags | All fields |
| 3.6.7 | `stream library delete --id 123` | Deletes |
| 3.6.8 | `stream video list --library-id 123` | Lists videos |
| 3.6.9 | `stream video list --library-id 123 --search "cat"` | Filtered |
| 3.6.10 | `stream video list` with all optional flags | Pagination, collection, order |
| 3.6.11 | `stream video get --library-id 123 --video-id abc-def` | Video details |
| 3.6.12 | `stream video upload --library-id 123 --file ./video.mp4` | Creates + uploads |
| 3.6.13 | `stream video upload --title "My Video" --collection-id xyz` | With optional params |
| 3.6.14 | `stream video upload` non-existent file | Helpful error |
| 3.6.15 | `stream video delete --library-id 123 --video-id abc` | Deletes |

### 3.7 Shield

| # | Test | Verify |
|---|------|--------|
| 3.7.1 | `shield zone list` | Lists all shield zones |
| 3.7.2 | `shield zone get --shield-zone-id 123` | Zone details |
| 3.7.3 | `shield zone get-by-pullzone --pull-zone-id 456` | Zone by pull zone |
| 3.7.4 | `shield zone create --pull-zone-id 456` | Creates shield zone |
| 3.7.5 | `shield zone update --shield-zone-id 123 --waf-enabled true` | Enables WAF |
| 3.7.6 | `shield zone update` all optional flags | DDoS, learning mode, etc. |
| 3.7.7 | `shield waf profiles` | Lists WAF profiles |
| 3.7.8 | `shield waf list-rules --shield-zone-id 123` | Lists WAF rules |
| 3.7.9 | `shield waf get-rule --id 456` | Rule details |
| 3.7.10 | `shield waf add-rule` with required flags | Creates rule |
| 3.7.11 | `shield waf add-rule` with all flags | Name, value included |
| 3.7.12 | `shield waf update-rule --id 456 --name "Updated"` | Updates |
| 3.7.13 | `shield waf delete-rule --id 456` | Deletes |
| 3.7.14 | `shield rate-limit list --shield-zone-id 123` | Lists rules |
| 3.7.15 | `shield rate-limit get --id 456` | Rule details |
| 3.7.16 | `shield rate-limit create` all required flags | Creates rule |
| 3.7.17 | `shield rate-limit create` with optional name/value | Full create |
| 3.7.18 | `shield rate-limit update --id 456 --name "Updated"` | Updates |
| 3.7.19 | `shield rate-limit delete --id 456` | Deletes |
| 3.7.20 | `shield access-list list --shield-zone-id 123` | Lists access lists |
| 3.7.21 | `shield access-list get --shield-zone-id 123 --id 456` | Access list details |
| 3.7.22 | `shield access-list create` all required flags | Creates |
| 3.7.23 | `shield access-list update` optional fields | Updates |
| 3.7.24 | `shield access-list delete --shield-zone-id 123 --id 456` | Deletes |
| 3.7.25 | `shield access-list update-config` | Updates config |
| 3.7.26 | `shield bot-detection get --shield-zone-id 123` | Detection settings |
| 3.7.27 | `shield bot-detection update` all flags | Updates all settings |

### 3.8 Script

| # | Test | Verify |
|---|------|--------|
| 3.8.1 | `script list` | Lists scripts |
| 3.8.2 | `script list --search "worker"` | Filtered |
| 3.8.3 | `script get --id 123` | Script details |
| 3.8.4 | `script create --name test --script-type 1 --create-linked-pull-zone true` | Creates |
| 3.8.5 | `script create` with --code and --linked-pull-zone-name | Full create |
| 3.8.6 | `script update --id 123 --name "New Name"` | Updates |
| 3.8.7 | `script delete --id 123 --delete-linked-pull-zones false` | Deletes |
| 3.8.8 | `script delete --id 123 --delete-linked-pull-zones true` | Deletes with zones |
| 3.8.9 | `script publish --id 123` | Publishes |
| 3.8.10 | `script publish --id 123 --note "v1.0"` | Publishes with note |
| 3.8.11 | `script statistics --id 123` | Shows stats |
| 3.8.12 | `script statistics --id 123 --date-from 2026-01-01 --date-to 2026-01-31 --hourly true` | Date range + hourly |
| 3.8.13 | `script code get --id 123` | Shows code |
| 3.8.14 | `script code update --id 123 --code "console.log('hi')"` | Updates inline |
| 3.8.15 | `script code update --id 123 --file ./script.js` | Updates from file |
| 3.8.16 | `script code update --id 123 --code X --file Y` | Error: mutually exclusive |
| 3.8.17 | `script release list --id 123` | Lists releases |
| 3.8.18 | `script release get-active --id 123` | Active release |
| 3.8.19 | `script variable list --id 123` | Lists variables (via get_script) |
| 3.8.20 | `script variable add --id 123 --name MY_VAR --required true` | Adds variable |
| 3.8.21 | `script variable add` with --default-value | With default |
| 3.8.22 | `script variable update --id 123 --variable-id 456 --required false` | Updates |
| 3.8.23 | `script variable delete --id 123 --variable-id 456` | Deletes |
| 3.8.24 | `script variable upsert --id 123 --name MY_VAR` | Upserts |
| 3.8.25 | `script secret list --id 123` | Lists secrets (names only, no values) |
| 3.8.26 | `script secret add --id 123 --name API_KEY --value secret123` | Adds |
| 3.8.27 | `script secret update --id 123 --secret-id 456 --value newsecret` | Updates |
| 3.8.28 | `script secret delete --id 123 --secret-id 456` | Deletes |
| 3.8.29 | `script secret upsert --id 123 --name API_KEY --value secret` | Upserts |

### 3.9 Container

| # | Test | Verify |
|---|------|--------|
| 3.9.1 | `container app list` | Lists apps |
| 3.9.2 | `container app list --cursor abc --limit 5` | Pagination |
| 3.9.3 | `container app get --id abc-123` | App details |
| 3.9.4 | `container app create --name myapp --runtime-type Shared --min 1 --max 3 --region eu-west` | Creates |
| 3.9.5 | `container app create` with multiple --region flags | Multiple regions |
| 3.9.6 | `container app update --id abc --name "New Name"` | Updates (PATCH) |
| 3.9.7 | `container app deploy --id abc` | Deploys |
| 3.9.8 | `container app undeploy --id abc` | Undeploys |
| 3.9.9 | `container app restart --id abc` | Restarts |
| 3.9.10 | `container app delete --id abc` | Deletes |
| 3.9.11 | `container app overview --id abc` | Shows overview with pods |
| 3.9.12 | `container app statistics --id abc --from 2026-01-01` | Stats |
| 3.9.13 | `container app statistics --id abc --from 2026-01-01 --to 2026-01-31 --granularity Hourly` | Full stats |
| 3.9.14 | `container app autoscaling-get --app-id abc` | Gets autoscaling config |
| 3.9.15 | `container app autoscaling-update --app-id abc --min 2 --max 5` | Updates autoscaling |
| 3.9.16 | `container app region-settings-get --app-id abc` | Gets region settings |
| 3.9.17 | `container app region-settings-update --app-id abc --allowed-region eu-west --required-region eu-central` | Updates regions |
| 3.9.18 | `container template get --app-id abc --container-id def` | Gets template |
| 3.9.19 | `container template add` all required flags | Creates template |
| 3.9.20 | `container template update --app-id abc --container-id def --image-tag v2` | Updates |
| 3.9.21 | `container template delete --app-id abc --container-id def` | Deletes |
| 3.9.22 | `container template env --app-id abc --container-id def --env KEY1=val1 --env KEY2=val2` | Sets env vars |
| 3.9.23 | `container endpoint list --app-id abc` | Lists endpoints |
| 3.9.24 | `container endpoint add` with --cdn | CDN endpoint |
| 3.9.25 | `container endpoint add` with --anycast | Anycast endpoint |
| 3.9.26 | `container endpoint add` with both --cdn and --anycast | Error: mutually exclusive |
| 3.9.27 | `container endpoint update` | Updates endpoint |
| 3.9.28 | `container endpoint delete --app-id abc --endpoint-id def` | Deletes |
| 3.9.29 | `container volume list --app-id abc` | Lists volumes |
| 3.9.30 | `container volume update --app-id abc --volume-id def --size 10` | Updates |
| 3.9.31 | `container volume detach --app-id abc --volume-id def` | Detaches |
| 3.9.32 | `container volume delete --app-id abc --volume-id def` | Deletes all instances |
| 3.9.33 | `container volume delete-instance --app-id abc --volume-id def --instance-id ghi` | Single instance |
| 3.9.34 | `container registry list` | Lists registries |
| 3.9.35 | `container registry get --id 123` | Registry details |
| 3.9.36 | `container registry create --name "Docker Hub"` | Creates |
| 3.9.37 | `container registry create` with type/username/password | Full create |
| 3.9.38 | `container registry update --id 123 --name "Updated"` | Updates |
| 3.9.39 | `container registry delete --id 123` | Deletes |
| 3.9.40 | `container registry image-tags` | Lists tags |
| 3.9.41 | `container registry image-digest` | Gets digest |
| 3.9.42 | `container registry config-suggestions` | Gets suggestions |
| 3.9.43 | `container registry search-public --registry-id 1 --query nginx` | Searches |
| 3.9.44 | `container registry search-public` with --size and --page | Pagination |
| 3.9.45 | `container region list` | Lists regions |
| 3.9.46 | `container region optimal` | Gets optimal region |
| 3.9.47 | `container node list` | Lists nodes |
| 3.9.48 | `container pod recreate --app-id abc --pod-id def` | Recreates pod |
| 3.9.49 | `container limits` | Shows account limits |
| 3.9.50 | `container log-forwarding list` | Lists configs |
| 3.9.51 | `container log-forwarding get --app-id abc` | Gets config |
| 3.9.52 | `container log-forwarding create` all required flags | Creates |
| 3.9.53 | `container log-forwarding create` with --token | With auth |
| 3.9.54 | `container log-forwarding update` all fields | Updates |
| 3.9.55 | `container log-forwarding delete --app-id abc` | Deletes |

### 3.10 Completions

| # | Test | Verify |
|---|------|--------|
| 3.10.1 | `completions bash` | Valid bash completion script |
| 3.10.2 | `completions zsh` | Valid zsh completion script |
| 3.10.3 | `completions fish` | Valid fish completion script |
| 3.10.4 | `completions powershell` | Valid powershell completion script |

---

## Part 4: Error Handling Tests

| # | Test | How | Expected |
|---|------|-----|----------|
| 4.1 | Missing API key | Unset BUNNY_API_KEY | Clear error message |
| 4.2 | Invalid API key (401) | Use wrong key | "Unauthorized" or similar |
| 4.3 | Not found (404) | Get non-existent resource | "Not found" error |
| 4.4 | Validation error (400) | Send invalid data | Error with details |
| 4.5 | Server error (500) | Mock 500 response | "Server error" message |
| 4.6 | Network timeout | Mock slow/no response | Timeout error |
| 4.7 | Missing required flag | Omit --id on get | Clap error with usage |
| 4.8 | Invalid flag value | `--id not-a-number` (for i64 IDs) | Parse error |
| 4.9 | Mutually exclusive flags | `--cdn true --anycast true` on endpoint | Error |
| 4.10 | File not found (upload) | `storage upload --file /nonexistent` | OS error |
| 4.11 | Permission denied (file) | Upload from unreadable file | OS error |
| 4.12 | Empty list response | API returns empty array | Empty table or "No results" |
| 4.13 | Confirmation denied | Answer "no" to delete prompt | Operation cancelled |
| 4.14 | Non-zero exit code on error | Any error scenario | Exit code != 0 |

---

## Part 5: Output Formatting Tests

| # | Test | How | Expected |
|---|------|-----|----------|
| 5.1 | JSON output is valid | `--format json` on all list commands | Valid JSON, parseable by jq |
| 5.2 | Table headers are correct | `--format table` on list commands | Correct column names |
| 5.3 | Table alignment | Visual check | Columns align properly |
| 5.4 | Long values truncated | Table with long URLs/names | No broken layout |
| 5.5 | Boolean display | Fields like waf_enabled | Shows true/false not 0/1 |
| 5.6 | Date formatting | Date fields in output | Consistent format |
| 5.7 | ID formatting | i64 and String IDs | No truncation or wrapping |
| 5.8 | Empty fields | Optional fields that are null | Shows "-" or empty, not "null" |
| 5.9 | Nested data (JSON) | Complex responses | Properly nested JSON |
| 5.10 | Text format | `--format text` | Clean key: value pairs |

---

## Part 6: API Client Unit Tests (Existing Coverage Check)

### 6.1 Already Covered (193 integration tests)

| Crate | Tests | Coverage |
|-------|-------|----------|
| bunny-api-core | 54 | Pull zones (partial), storage zones, DNS, video libraries, billing |
| bunny-api-containers | 57 | All endpoints covered |
| bunny-api-shield | 27 | All endpoints covered |
| bunny-api-storage | 9 | All endpoints covered |
| bunny-api-stream | 18 | Videos + collections |
| bunny-api-compute | 28 | All endpoints covered |

### 6.2 Missing API Client Tests

| # | Crate | Missing Test | Priority |
|---|-------|-------------|----------|
| 6.2.1 | bunny-api-core | `create_pull_zone` | High |
| 6.2.2 | bunny-api-core | `update_pull_zone` | High |
| 6.2.3 | bunny-api-core | `delete_pull_zone` | High |
| 6.2.4 | bunny-api-core | `purge_pull_zone_cache` | High |
| 6.2.5 | bunny-api-core | `purge_pull_zone_cache` with tag | Medium |
| 6.2.6 | bunny-api-stream | `fetch_video` | Medium |
| 6.2.7 | bunny-api-compute | `rotate_deployment_key` | Medium |

---

## Part 7: Edge Cases & Regression Tests

| # | Test | Details |
|---|------|---------|
| 7.1 | Unicode in names | Create resource with Unicode characters |
| 7.2 | Special chars in values | DNS TXT record with quotes, semicolons |
| 7.3 | Very long values | 1000+ char string in a field |
| 7.4 | Zero-value IDs | `--id 0` behavior |
| 7.5 | Negative IDs | `--id -1` behavior |
| 7.6 | Max pagination | `--per-page 99999` |
| 7.7 | Empty string values | `--name ""` |
| 7.8 | Multiple --region flags | `container app create --region a --region b --region c` |
| 7.9 | Multiple --env flags | `container template env --env A=1 --env B=2` |
| 7.10 | Env var with = in value | `--env KEY=val=ue` |
| 7.11 | Large file upload | Upload 1GB+ file to storage |
| 7.12 | Binary file download | Download binary to stdout vs file |
| 7.13 | Concurrent requests | Rapid sequential commands |
| 7.14 | Storage region routing | Each region prefix maps to correct hostname |
| 7.15 | Stream API key per library | Stream uses library API key, not account key |

---

## Part 8: Test Execution Strategy

### Phase 1: Automated (Can Run in CI)
- **Help text tests (Part 1):** Build binary, run `--help` for every command, parse output
- **Error handling (Part 4):** Most testable with wiremock
- **Output formatting (Part 5):** Snapshot tests against known fixtures
- **API client tests (Part 6):** Already exist, just run `cargo test`

### Phase 2: Semi-Automated (Wiremock CLI Tests)
- **Command functional tests (Part 3):** New integration test suite using wiremock
- **Edge cases (Part 7):** Mix of unit and integration tests

### Phase 3: Manual / Live API (Optional)
- **Live smoke tests:** Run key commands against real Bunny.net staging/test account
- **Upload/download tests:** Real file transfers
- **End-to-end workflows:** Create zone → add record → verify → delete

### Priority Order
1. Help text accuracy (Part 1) — fast, catches documentation bugs
2. Missing API client tests (Part 6.2) — fill pull-zone CRUD gap
3. Global flag tests (Part 2) — ensures consistent UX
4. Error handling (Part 4) — user-facing quality
5. Command functional tests (Part 3) — bulk of the work
6. Edge cases (Part 7) — hardening

---

## Test Count Summary

| Part | Category | Test Count |
|------|----------|-----------|
| 0 | Gap analysis | 10 items |
| 1 | Help text accuracy | ~120 tests |
| 2 | Global flags | 10 tests |
| 3 | Command functional | ~180 tests |
| 4 | Error handling | 14 tests |
| 5 | Output formatting | 10 tests |
| 6 | API client gaps | 7 tests |
| 7 | Edge cases | 15 tests |
| **Total** | | **~356 new tests** |
| Existing | API client wiremock | 193 tests |
| **Grand Total** | | **~549 tests** |

## Related
- [[development-roadmap]] — roadmap and current test architecture
- [[adding-a-feature]] — feature checklist including test steps
- [[iterations/rust-e2e-rewrite-plan]] — E2E test architecture plan
- [[testing/test-environment-needed]] — test environment requirements
- [[testing/e2e-test-report]] — latest test results
