# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking

- **WAF operator labels were wrong and are now fixed.** Wire numbers are
  unchanged, only the names. `--operator-type` (`shield waf add-rule` /
  `update-rule`, `shield rate-limit create` / `update`) now takes the shield
  spec names (case-insensitive) or numbers: `0 BEGINSWITH, 1 ENDSWITH,
  2 CONTAINS, 3 CONTAINSWORD, 4 STRMATCH, 5 EQ, 6 GE, 7 GT, 8 LE, 9 LT,
  12 WITHIN, 13 IPMATCH, 14 RX, 15 STREQ, 17 DETECTSQLI, 18 DETECTXSS`.
  **Migration:** if you chose an operator by the old help text, re-check the
  number: old `0 = Eq` was really BEGINSWITH, old `6 = Regex` was GE, old
  `14 = PmFromFile` is RX, old `15 = IpMatch` is STREQ (IPMATCH is `13`), and
  `17`/`18` are DETECTSQLI/DETECTXSS. Old names are not accepted as aliases
  (`eq` now means `5`). When `shield waf update-rule` has to invent a
  configuration for a rule that has none, it now defaults to EQ (5) instead
  of 0.
- `bunny-net-api` library: `WafRuleOperatorType` variants renamed to the spec
  names (`BeginsWith`, `EndsWith`, `Contains`, `ContainsWord`, `StrMatch`,
  `Eq`, `Ge`, `Gt`, `Le`, `Lt`, `Within`, `IpMatch`, `Rx`, `StrEq`,
  `DetectSqli`, `DetectXss`); `DdosShieldSensitivity::Disabled` / `VeryHigh`
  became `Off` / `Challenge`; `BotCategorizationGroup.category_action` and
  `BotCategoryDetails.action` use the new `BotCategoryAction` (no `Ignore`);
  `set_bot_category_action` takes `BotCategoryAction`;
  `EdgeScriptExecutionPhase::OnRequest` / `OnResponse` became `Cache` /
  `LoadBalancer` (+ `PreCache`); `StorageRegion` has `code` / `endpoint` /
  `s3_endpoint` / ... instead of `id` / `url`; `DnsMonitoringStatus::Unknown`
  is now `Undetermined`; `ShieldZoneRequest.d_do_s_execution_mode` is gone;
  `cleanup_video_resolutions` returns `ResolutionsCleanupStatus`;
  `WafRuleConfiguration`, `RateLimitRuleConfiguration` and
  `WafChainedRuleCondition` gained `is_negated` / `is_regex_variable`.
- Every integer enum in `bunny-net-api` is now open: it has an `Unknown(n)`
  variant (so exhaustive `match`es need an extra arm), and
  `core::serde_helpers::deserialize_repr_option` was removed. Unknown values
  now round-trip instead of becoming `None`.
- `hoppy shield zone` table: DDoS sensitivity `Disabled` / `VeryHigh` are now
  shown as `Off` / `Challenge`. `storage-zone regions` columns are now Code,
  Name, Endpoint, S3 Endpoint (JSON keys `Id` / `Url` no longer exist).
  `stream video resolutions cleanup --format json` gains a `Data` key.
- `pull-zone update --edge-script-execution-phase` accepts `cache`,
  `load-balancer`, `pre-cache`; `on-request` / `on-response` still work as
  hidden aliases for the same wire values 0 / 1.
- `shield zone update --ddos-execution-mode` is deprecated and ignored (hidden,
  prints a warning): the API no longer accepts it.
- **Shield enum labels now follow the spec (wire numbers unchanged).** The old
  labels were wrong for the numbers they named. The worst case: access-list
  action `1` is **Allow** and `2` is **Block** (hoppy had them swapped, so a
  policy picked by label was inverted). The API's own
  `shield access-list enums` agrees with the spec. The flags below now take a
  spec name (case-insensitive) or a number; old labels are **not** accepted as
  aliases, because most would now mean a different number. Check any script
  that passes a number chosen from the old help text, and any code matching on
  the old variant names. Migration (number: old label to new label):

  | Enum / flag | Migration |
  |---|---|
  | `AccessListAction` (`access-list update-config --action`) | 0 NoAction to None, 1 Block to **Allow**, 2 Allow to **Block**, 3 LogOnly to Challenge, 4 Challenge to Log, 5 ChallengeInterstitial to Bypass |
  | `AccessListType` (`access-list create --type`) | 0 Ip to IP, 1 Country to **CIDR**, 2 Asn to ASN, 3 Hostname to **Country**, 4 UserAgent to Organization, 5 Custom to JA4 |
  | `WafRuleActionType` (`waf add-rule/update-rule --action-type`) | 1 Block, 2 LogOnly to Log, 3 Challenge, 4 ChallengeInterstitial to **Allow**, 5 Allow to **Bypass** |
  | `RateLimitActionType` (`rate-limit create/update --action-type`) | 1 Block to RateLimit, 2 LogOnly to Log, 3 Challenge |
  | `WafRuleSeverityType` (`--severity-type`) | 0 Low to NOTICE, 1 Medium to WARNING, 2 High to CRITICAL |
  | `ShieldPlanType` | 0 Basic, 1 Standard to **Advanced**, 2 Advanced to **Business**, 3 Enterprise |
  | `WafExecutionMode` (`zone update --waf-execution-mode`) | 0 Disabled to Log, 1 Enabled to Block |
  | `DdosExecutionMode` | 0 Disabled to Log, 1 Enabled to Block |
  | `BotDetectionExecutionMode` (`bot-detection update --execution-mode`) | 0 Disabled to LogOnly, 1 Enabled to Challenge |
  | `BotDetectionSensitivity` (`--*-sensitivity`) | 0 Disabled to Off, 1-3 Low/Medium/High unchanged |
  | `BrowserFingerprintAggression` (`--fingerprint-aggression`) | 0 Disabled to Unknown, 1 VeryLow to Low, 2 Low to Medium, 3 Medium to High, 4 High to Custom |
  | `RateLimitCounterKey` (`rate-limit --counter-key-type`) | 0 Global to IP, 1 PerIp to Host, 2 PerCountry to Country, 3 PerAsn to City, 4 PerHeader to ASN, 5 PerCookie to Organization, 6 PerQuery to JA4, 7 PerFingerprint to IP_JA4 |
  | `WafRuleTransformationType` (`--transformation-type`) | the whole table changed; numbers to re-pick by intent (old label: old number to new number): Lowercase 1 to **8**, UrlDecode 3 to **19**, UrlDecodeUni 4 to **20**, JsDecode 9 to **6**, CssDecode 10 to **3**, CompressWhitespace 20 to **2**, NormalisePath 14 to **11**, NormalisePathWin 15 to **13**, RemoveComments 16 to **14**, RemoveNulls 17 to **15**, RemoveWhitespace 18 to **16**, ReplaceComments 19 to **17**, HtmlEntityDecode 5 unchanged. No spec equivalent any more: Uppercase, Base64Decode, Base64DecodeExt, Base64Encode, Trim, TrimLeft, TrimRight, None. New in the spec: CMDLINE 1, HEXENCODE 4, LENGTH 7, MD5 9, NORMALIZEPATH 10, NORMALIZEPATHWIN 12, SHA1 18, UTF8TOUNICODE 21. Old numbers 1, 2, 3, 4, 6-13, 15-21 now mean something else, so a stored `--transformation-type 1` is CMDLINE, not Lowercase |
  | `ReviewActionType` (`waf review-triggered-rule --action`) | 0 Pending to Ignore, 1 Approve to LogOnly, 2 Reject to DisableRule |
  | `WafPayloadLimitAction` | 0 Allow to Block, 1 Block to Log, 2 Ignore |
  | `UploadScanningScannerMode` (`upload-scanning update --*-mode`) | 0 Disabled, 1 LogOnly to Log, 2 Block |

  **JSON output labels changed meaning.** The list commands print string
  labels in `--format json` (`access-list list`: `"action": "Block"`,
  `"list_type": "Country"`; `shield zone list`: `"plan_type"`,
  `"ddos_sensitivity"`; `waf list-rules`: `"action_type"`). Those strings now
  come from the spec tables, so for the same wire value they are different
  words: a script matching `.action == "Block"` on `access-list list` used to
  select wire 1 and now selects wire 2, and `"Country"` used to be wire 1 and
  is now wire 3. These rows now also carry the wire number next to each label,
  for scripts to match on instead: `access-list list` (`action_id`,
  `list_type_id`), `shield zone list` (`plan_type_id`,
  `ddos_sensitivity_id`), `waf list-rules` (`action_type_id`). Commands that
  print the raw API object (`access-list get`, `waf get-rule`,
  `shield zone get`, `bot-detection get`, `upload-scanning get`) already emit
  numbers and are unaffected. `bot-categorization set-bot` / `set-category
  --action` also take names now (`none`, `block`, `allow`, `ignore`; numbers
  0-3 / 0-2 as before).

  **Library (`bunny-net-api`) variant renames** (this needs a minor version
  bump; the crate is pre-1.0 so it is a 0.x minor):

  - `AccessListAction`: `NoAction` to `None`, `Block` to `Allow` (wire 1),
    `Allow` to `Block` (wire 2), `LogOnly` to `Challenge` (3), `Challenge` to
    `Log` (4), `ChallengeInterstitial` to `Bypass` (5).
  - `AccessListType`: `Ip` stays, `Country` (1) to `Cidr`, `Asn` stays,
    `Hostname` (3) to `Country`, `UserAgent` (4) to `Organization`, `Custom`
    (5) to `Ja4`.
  - `WafRuleActionType`: `LogOnly` to `Log`, `ChallengeInterstitial` (4) to
    `Allow`, `Allow` (5) to `Bypass`.
  - `RateLimitActionType`: `Block` to `RateLimit`, `LogOnly` to `Log`.
  - `WafRuleSeverityType`: `Low` / `Medium` / `High` to `Notice` / `Warning` /
    `Critical`.
  - `ShieldPlanType`: `Standard` (1) to `Advanced`, `Advanced` (2) to
    `Business`.
  - `WafExecutionMode`, `DdosExecutionMode`: `Disabled` / `Enabled` to `Log` /
    `Block`.
  - `BotDetectionExecutionMode`: `Disabled` / `Enabled` to `LogOnly` /
    `Challenge`.
  - `BotDetectionSensitivity`: `Disabled` to `Off`.
  - `BrowserFingerprintAggression`: `Disabled` to `Unspecified` (spec
    `Unknown`), `VeryLow` removed, `Low` (2) to `Medium`, `Medium` (3) to
    `High`, `High` (4) to `Custom`, and `Low` is now wire 1.
  - `RateLimitCounterKey`: `Global`, `PerIp`, `PerCountry`, `PerAsn`,
    `PerHeader`, `PerCookie`, `PerQuery`, `PerFingerprint` to `Ip`, `Host`,
    `Country`, `City`, `Asn`, `Organization`, `Ja4`, `IpJa4` (wire 0 to 7).
  - `WafRuleTransformationType`: now `CmdLine`, `CompressWhitespace`,
    `CssDecode`, `HexEncode`, `HtmlEntityDecode`, `JsDecode`, `Length`,
    `Lowercase`, `Md5`, `NormalizePath`, `NormalisePath`, `NormalizePathWin`,
    `NormalisePathWin`, `RemoveComments`, `RemoveNulls`, `RemoveWhitespace`,
    `ReplaceComments`, `Sha1`, `UrlDecode`, `UrlDecodeUni`, `Utf8ToUnicode`
    (wire 1 to 21 in that order); `Uppercase`, `Base64Decode`,
    `Base64DecodeExt`, `Base64Encode`, `Trim`, `TrimLeft`, `TrimRight` and
    `None` are gone.
  - `ReviewActionType`: `Pending` / `Approve` / `Reject` to `Ignore` /
    `LogOnly` / `DisableRule`.
  - `WafPayloadLimitAction`: `Allow` (0) to `Block`, `Block` (1) to `Log`,
    `Ignore` stays.
  - `UploadScanningScannerMode`: `LogOnly` to `Log`.
  - Every enum above gains `NAMES`, `name()`, `describe_all()`, `Display` and
    `FromStr` (spec name, case-insensitive, or declared number);
    `BotCategorizationAction` and `BotCategoryAction` gain them too.
    `WafRuleOperatorType` now gets them from the same macro.
  - Because wire numbers did not move but meanings did, compiled code that
    matched on the old variants keeps compiling in many cases (same variant
    name, different number) and silently changes behaviour; review every use.

  Remaining non-Shield label mismatches (pull-zone watermark position,
  preloading theme, ...) are tracked in
  `hoppy-knowledgebase/backlog/core-enum-labels-disagree-with-spec.md`.

### Added

- Open enums: an unrecognised value from the API no longer fails a whole list
  or get response. It is shown as `Unknown(n)` and sent back unchanged by
  read-modify-write commands (`waf update-rule`, `rate-limit update`).
- `--ddos-sensitivity 5` (ChallengeInteractive), `bot-categorization set-bot
  --action 3` (Ignore), volume status `maintenance`, registry type
  `bunnyRegistry` (read), edge-rule actions 35-37 (`use-load-balancer`,
  `force-load-balancer-sticky-session-mode`, `strip-cookies-client-to-origin`).
- `stream video resolutions cleanup` prints the cleanup plan (resolutions,
  storage objects, playlist changes), which is the main `--dry-run` output.
- WAF `isNegated` / `isRegexVariable` are read and written back by
  `update-rule` instead of being dropped.
- `specs/` refreshed to the 2026-10-23 bunny.net OpenAPI documents, plus a
  `partner-portal.json` snapshot (no client yet).

### Changed

- **Drill-down `tip:` hints are now printed on stderr under `--format json`
  too** (previously `--format json` implied `--no-hints`). stdout stays pure
  JSON; `--no-hints` / `--quiet` still suppress them. **Migration:** scripts
  that merge stderr into stdout (`2>&1`) while parsing JSON should pass
  `--no-hints`. Tips that were always emitted but silenced by the JSON
  implication now appear: `pull-zone create`, `container app create`, the `auth`
  command's `hoppy pull-zone list` tip. Tips that were gated on
  table output now appear too: `pull-zone list`, `dns zone scan start`,
  `search` pagination, `country list`, `billing payment-requests`, `stream
  library list`, `user audit` continuation, and the `--reveal` tips of
  `apikey list` / `billing records`. "use --format json" truncation tips and
  the `statistics --hourly` tip stay table-only.
- The macOS release binaries are signed with a Developer ID certificate
  and notarised (ractive/release-workflows v0.2.4).

## [0.7.0] - 2026-08-13

### Added

- `hoppy billing records`: lists individual billing records (invoices,
  top-ups, refunds, coupon credits, …) from `GET /billing`'s `BillingRecords`
  array, which `BillingDetails` previously dropped entirely. Shows id,
  timestamp, amount, type (friendly name, falling back to the raw number for
  unrecognised future values), payer, and invoice availability. `payer` is
  redacted by default (`--reveal` to show it); the pre-signed
  `DocumentDownloadUrl` is likewise redacted in `--format json`.
- `hoppy billing receipt-pdf --record-id <ID> --output <file>`: downloads a
  billing record's document via its pre-signed `DocumentDownloadUrl`. Covers
  record types — top-ups in particular — for which `billing invoice-pdf`
  (`GET /billing/summary/{id}/pdf`) 404s, since their PDF only exists behind
  the signed URL. The request to that URL (a different host,
  `billing.b-cdn.net`) never carries the `AccessKey` header, and the URL's
  `token` query parameter is never logged, even under `--debug`.

### Changed

- `hoppy billing invoice-pdf`: on a 404, the error message now explains that
  the record likely has no formal invoice (e.g. it's a top-up/payment
  receipt) and points at `hoppy billing receipt-pdf` instead.
- `hoppy db group create`: `--storage-region`, `--primary-region`, and
  `--replicas-region` are now validated case-sensitively against the live
  vocabulary from `GET /v1/config` before the create request is sent. An
  unknown value fails locally with the valid-value list; a casing-only
  mismatch (e.g. `de` instead of `DE`) gets a did-you-mean. No region
  vocabulary is hardcoded — the check always reflects the current API. The
  config fetch is read-only, so it still runs (and stays truthful) under
  `--dry-run`. Help text for all three flags now names the vocabulary shape
  with examples and points at `hoppy db config show` for the full list.

### Fixed

- `hoppy db create --slug` / `hoppy db fork --target`: the local slug-length
  limit was wrong (24 chars) and didn't actually prevent the upstream
  `HTTP 500 "Internal error"` it claimed to guard against. Live-measured
  against the real API on 2026-08-13: a 16-char slug creates fine; 17, 18,
  and 19 chars all 500. The limit is now 16
  (`^[a-z][a-z0-9-]{0,15}$`), and the error/help text reflect the real
  boundary.
- `--reveal`: threaded into every remaining `auth::core_client` call site
  that was building `ClientOpts` via `..Default::default()`, which silently
  pinned `reveal_secrets: false` regardless of the flag. Affected commands:
  `auth check`, `billing summary`/`payment-requests`/`*-pdf`,
  `region list`, `country list`, `search`, `user audit`, `dns` (all
  subcommands), `purge`, `statistics`, `video-library`, and the nested
  pull-zone cleanup client in `container app delete --cascade`. Fails safe
  before this fix (nothing leaked), but `--debug --reveal` under-delivered
  on those commands, still masking secret-shaped fields in the HTTP body
  dump. `ClientOpts` no longer derives `Default`, so every construction
  must state `reveal_secrets` explicitly — the compiler now catches this
  class of bug at future call sites. The redundant `core_client_with_reveal`
  alias was removed in favor of a single `core_client(opts)`.

## [0.6.0] - 2026-08-09

### Added

- Global `--dry-run` flag: previews mutating (POST/PUT/PATCH/DELETE) API
  calls — method, URL, and (redacted, unless `--reveal`) body — without
  sending them. Read-only (GET/HEAD) requests still execute, so composite
  commands (e.g. resolving a storage zone's password, a stream library's
  key, or a container app's linked pull zones) stay truthful; only the
  first blocked mutation is previewed, not any follow-up requests.
  `--dry-run` implies `--yes` (confirmation prompts are skipped since the
  mutation never reaches the API regardless). `--format json` prints a
  `{"status":"dry-run",...}` envelope to stdout; `table`/`text` print an
  `[dry-run]`-prefixed preview to stderr.

### Changed

- `stream video resolutions cleanup`'s local `--dry-run` flag was folded
  into the new global `--dry-run` — it now drives the same server-side
  `?dryRun=true` preview as before, just via the global flag instead of a
  subcommand-local one.

## [0.5.0] - 2026-07-11

Release-infrastructure release: iterations 79 and 80 (scope A).

### Added

- musl static Linux builds (`x86_64-unknown-linux-musl`,
  `aarch64-unknown-linux-musl`). Homebrew on Linux now installs the static
  musl binaries instead of the glibc-linked ones.
- CycloneDX SBOMs and GitHub build-provenance attestations for native
  builds.
- crates.io publish recovery workflow (`publish-crates.yml`).

### Changed

- The release pipeline moved to the shared reusable workflow in
  [ractive/release-workflows](https://github.com/ractive/release-workflows)
  (`@v0.2.0`); `release.yml` is a thin caller. Releases can be rehearsed
  with a `workflow_dispatch` dry run.
- `.deb`/`.rpm` packages are now also published to the hosted apt/yum repos
  at Cloudsmith (`ractive/hoppy`).

### Fixed

- Cross-compiled aarch64 binaries embedded container-local git provenance
  in `hoppy -V`: `Cross.toml` now passes `GIT_COMMIT`/`GIT_COMMIT_DATE`
  into cross containers.
- Removed the vestigial `libssl-dev` cross pre-build: hoppy has been
  rustls-only for its whole dependency tree (no `openssl-sys`/`native-tls`
  in the lockfile).

## [0.4.0] - 2026-07-10

Covers iterations 33–78 (the May–July development run, including the full
July API-coverage gap-analysis plan).

### Added

- **`hoppy logs` command group** — retrieve CDN access logs and origin error
  logs (new `logging` and `origin-errors` API modules) (iter-70)
- **Resumable video uploads** — `stream video upload --resumable` via the TUS
  protocol; survives interruptions and process re-runs (iter-77)
- **Account & billing surface** — `apikey`, `billing` (summary, payment
  requests, invoice download), `region`, `country`, `search` (global
  cross-resource), `user` (account audit log) (iter-75)
- **Credential rotation** — rotate storage-zone passwords, stream library
  keys, and pull-zone security keys (iter-67)
- **DNS completeness** — smart-routing records, zone export formats,
  `dnssec`, scan-by-domain, certificate-issuance hints, remaining
  spec-only endpoints (iters 53, 58, 60–61, 71)
- **Shield expansion** — new surface coverage (API Guardian, bot
  categorization, custom pages), metrics flag parity, structured 202 error
  envelopes (iters 50, 54, 59, 72)
- **Video library settings** — ~35 new update flags (DRM, transcoding,
  player configuration) (iter-73)
- **Pull-zone completeness** — security/compliance toggles, vary headers +
  caching toggles, origin + routing toggles, firewall + rate limiting, full
  update-body coverage (iters 44–47, 74)
- **Magic Containers polish** — volumes, health probes (`--probes-json`),
  endpoints, registry image-config (iter-76)
- **Stream library API keys** — surfaced via CLI with `--reveal` opt-in
  (secrets stay redacted by default) (iter-52)
- **Filters & pagination sweep** — consistent `--page`/`--per-page`/filter
  flags and `--list-all` across list commands (iters 63, 69)
- **CLI discoverability** — drill-down hints after commands (`--no-hints` to
  suppress), `--quiet` contract, `--format` parity across all subcommands
  (iters 35, 51, 57)
- **Fixture tooling for contributors** — `--record <dir>` /
  `HOPPY_RECORD_DIR` response recording with PII redaction by default
  (`--no-redact` escape hatch), and the read-only `fixture-refresh
  --shape-report` API drift radar (iters 33–34, 48, 78)

### Fixed

- Seven API correctness drifts found by the July spec refresh: `db fork`
  payload, API Guardian `/spec` path, `cdn_server_token`, streaming storage
  downloads, base64 caption uploads, `{region}.storage.bunnycdn.com`
  endpoint casing, geo-zone serde casing (iters 49, 66)
- TUS resumable upload never resumed across process re-runs (caught in PR
  review) (iter-77)
- Record-mode PII redaction gaps: account API key under `Key`,
  `DeploymentKey`, `ZoneSecurityKey`, person-name fields, billing amounts;
  JWT false positive that redacted three-label hostnames
- Dozens of dogfooding fixes: negative-integer flag rejection, empty help
  strings, deterministic chart ordering, `db` output formats, DNS scan
  domain column, empty-zone exports (iters 39–42, 55–58, 62, 64–65)

### Changed

- Fixture values are test contracts: recorded live responses never
  overwrite `fixtures/` (`fixture-refresh --apply` removed) (iter-78)
- Shape-first wiremock asserts and drift-tolerant CLI snapshots make the
  offline suite robust against API value churn (iters 36–37)

### Security

- Dependency updates: `quinn-proto` ≥ 0.11.15 (RUSTSEC-2026-0185, remote
  memory exhaustion, high), `anyhow` 1.0.103 (RUSTSEC-2026-0190
  unsoundness). `RUSTSEC-2026-0173` (`proc-macro-error2` unmaintained,
  compile-time only via `tabled`) documented as ignored in `deny.toml` — no
  fixed upstream exists yet.

## [0.3.0] - 2026-05-11

Released without a changelog entry; see the git history between `v0.2.0`
and `v0.3.0`. Highlights: consolidated `bunny-net-api` crate (iter-32) and
the e2e test-binary consolidation (iter-22).

## [0.2.0] - 2026-05-10

Released without a changelog entry; see the git history between `v0.1.1`
and `v0.2.0`. Highlights: Bunny Database (libSQL) support (iter-20), Magic
Containers UX & cross-cutting secret redaction (iter-21), `container logs`
syslog tunnel (iter-24), pull-zone edge rules and optimizer flags
(iters 26–29).

## [0.1.1] - 2026-05-09

### Release pipeline (iter-25)

- SHA-pinned GitHub Actions in `ci.yml` and `release.yml`
- `version-check` job: release tag must match `Cargo.toml` version
- `security` job: `cargo audit` + `cargo deny check` before any build
- `crates-io` job: publishes all 9 crates in dependency order with index-propagation retry
- `scoop` job: updates `ractive/scoop-bucket` Scoop manifest (`bucket/hoppy.json`) on each release
- Homebrew formula updated with macOS x86 target, caveats for optional `bore` dependency
- `deny.toml` added for license and advisory gating
- All crate `Cargo.toml` files: added `homepage`, `keywords`, `categories`, `readme`; internal workspace deps now carry `version = "0.1.1"` for crates.io compatibility
- Minimal `README.md` added to each sub-crate

### Magic Containers — container logs tunnel (iter-24)

- `hoppy container logs` — streams Magic Containers syslog via an in-process RFC 5424/3164 receiver
- Auto-tunnel via [bore](https://github.com/ekzhang/bore): zero-config public ingress on a kernel-assigned port
- `--tunnel none` / `--tunnel-host host:port` for custom ingress (SSH reverse-forward or private bore server)
- `--bore-server` flag to point at a self-hosted bore relay
- `bunny-syslog-receiver` crate extracted as a reusable, independently publishable library
- NDJSON output with `--format json`; `--format table` rejected (logs are not tabular)

### Project hygiene & dogfooding (iter-23)

- Cargo workspace hygiene: `[workspace.dependencies]`, `[workspace.package]`, `resolver = "3"`, optimised `[profile.release]`, `unsafe_code = "forbid"`
- `AI_NOTICE`, dogfooding playbook, CLI command tree, help-text style guide

### Stream — video processing (iter-16)

- `stream video transcribe`, `heatmap`, `reencode`, `repackage`, `smart-generate`, `set-thumbnail`, `resolutions list/cleanup`, `storage`

### Magic Containers — UX & safety (iter-21)

- Granular env-var mutation (`--add`, `--remove`, `--update`, `--replace-all`, `--clear`, `--list`)
- `container app delete --cascade` / `--no-cascade` to control Pull Zone orphaning
- `container app create` returns full document by default; `--minimal` for legacy shape
- `container app create --env KEY=VAL` for initial env in one call
- Secret redaction: env values masked as `<set, length=N>`; `--reveal` / `--reveal-env KEY` to opt in
- Confirmation phrases required for destructive env operations

## [0.1.0] - 2026-03-18

Initial release.

### Services

- **CDN Pull Zones** — list, get, create, update, delete, purge cache (by tag or full)
- **Storage Zones** — list, get, create, update, delete
- **Storage Files** — upload, download, list, delete with progress bars
- **DNS** — zone CRUD, record management for all record types (A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA, and bunny-specific types)
- **Video Streaming** — library CRUD, video list/get/upload/delete with progress bars
- **Shield (Security)** — zone management, WAF rules, rate limiting, access lists, bot detection, DDoS configuration
- **Edge Scripting** — script CRUD, publish, code get/update, releases, variables, secrets, statistics
- **Magic Containers** — applications, templates, endpoints, volumes, registries, regions, nodes, pods, limits, log forwarding
- **Auth** — API key validation with billing/account info

### Features

- Three output formats: `--format json|table|text`
- Debug mode (`--debug`) showing HTTP request details
- Quiet mode (`--quiet`) suppressing non-essential output
- Confirmation prompts for destructive operations (`--yes` to skip)
- Progress bars for file uploads and downloads
- Shell completions for bash, zsh, and fish
- Pagination support across all list commands
- Credentials excluded from JSON output for security

[Unreleased]: https://github.com/ractive/hoppy/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/ractive/hoppy/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ractive/hoppy/releases/tag/v0.1.0
