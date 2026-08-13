---
title: Iter-83 — billing records list + receipt PDF download
type: iteration
date: 2026-08-13
tags:
  - iteration
  - api-coverage
  - cli
  - billing
status: in-progress
branch: iter-83/billing-records
---

# Iter-83 — billing records list + receipt PDF download

## Why

External feedback (dogfooding by `@../comparis/credit-card-expenses`,
2026-08-13, live-verified against a real account): `GET /billing` returns a
`BillingRecords` array that `BillingDetails` silently drops. As a result no
hoppy command reveals billing-record IDs, which makes the existing
`hoppy billing invoice-pdf --record-id <ID>` (iter-75) nearly unusable —
and top-up receipts (record `Type: 2`) cannot be downloaded at all, because
`/billing/summary/{id}/pdf` returns **404** for them; their PDF exists only
behind the pre-signed `DocumentDownloadUrl`.

The spec already models this: `BillingRecordModel` and `BillingRecordType`
in `specs/core-platform.json`, and `fixtures/core/billing_get.json` already
contains a redacted `BillingRecords` array (2× Type 3, 1× Type 2) — only the
Rust type drops it.

## Verified API behavior (live, 2026-08-13)

- `BillingRecordModel` fields: `Id` (i64), `PaymentId` (string, nullable),
  `Amount` (f64), `Payer` (string, nullable; live also shows `""`),
  `Timestamp` (string, **no timezone suffix**, e.g. `2026-08-01T00:13:38`),
  `Type` (int enum), `InvoiceAvailable` (bool), `DocumentDownloadUrl`
  (string, nullable, pre-signed `billing.b-cdn.net` URL with
  `token`/`expires` query params, no auth header needed),
  `DetailedDocumentDownloadUrl` (string, nullable, points at
  `/billing/summary/{id}/pdf`).
- `BillingRecordType` per spec: PayPal = 0, Crypto = 1, CreditCard = 2,
  MonthlyUsage = 3, Refund = 4, CouponCode = 5, BankTransfer = 6,
  AffiliateCredits = 7. Unknown future values must not break
  deserialization or display.
- Type 2 (top-up): `InvoiceAvailable: false`,
  `DetailedDocumentDownloadUrl: null`, `/billing/summary/{id}/pdf` → 404;
  receipt PDF only via signed `DocumentDownloadUrl`.
- Type 3 (monthly usage invoice): `InvoiceAvailable: true`, PDF available
  via both routes.
- Signed-URL filenames are meaningful (`receipt_<id>.pdf` /
  `invoice_<id>.pdf`) — same names the dashboard produces.

## Design

### API crate (`crates/bunny-net-api/src/core/`)

- New `BillingRecord` struct in `types.rs`, PascalCase serde renames like
  its neighbors. `Type` deserializes as plain `i32` (tolerant of unknown
  values), `#[serde(rename = "Type")]` → field `record_type`. `Timestamp`
  stays a `String` (repo precedent: `date_created: String` elsewhere in
  `types.rs`; no chrono dependency for a display-only value). `PaymentId`,
  `Payer`, `DocumentDownloadUrl`, `DetailedDocumentDownloadUrl` are
  `Option<String>` (spec-nullable).
- `BillingDetails` gains
  `#[serde(rename = "BillingRecords", default)] pub billing_records: Vec<BillingRecord>`
  — `default` so older/partial responses keep parsing (same defensive style
  as the `AutomaticRechargeTreshold` misspelling handling in that struct).
- New client method `download_billing_record_document<W>(url: &str, out: &mut W)`
  (or equivalent) that streams a pre-signed document URL to a writer,
  mirroring `download_billing_invoice_pdf`'s streaming style.
  **Security-critical:** the pre-signed URL lives on `billing.b-cdn.net`,
  a different host — the request MUST NOT carry the `AccessKey` header
  (sending it would leak the API key to a CDN host), and the URL (its
  `token` param) must never be logged, cached, or included in error
  messages. Treat `token`/`expires` as opaque.

### CLI (`crates/hoppy-cli/`)

- `hoppy billing records` — new `BillingAction::Records`: calls
  `get_billing()`, renders the `billing_records` list. Columns: id,
  timestamp, amount, type, payer, invoice-available. Type shown as the
  spec's friendly name (`MonthlyUsage`, `CreditCard`, …) with unknown
  values displayed raw (e.g. `9`). Table + json + text formats like the
  other billing commands.
- Redaction: `Payer` (an email) is redacted by default and shown with
  `--reveal`, following the iter-75 `ApiKeyRow` / `redact::placeholder`
  precedent. Signed URLs are never shown in table/text; in `--format json`
  they are redacted like `redact_api_keys_json` does unless `--reveal`.
- `hoppy billing receipt-pdf --record-id <ID> --output <file>` — new
  `BillingAction::ReceiptPdf`: fetch `GET /billing`, find the record by id,
  stream its `DocumentDownloadUrl` to the file. Works for every record type
  that has a `DocumentDownloadUrl` (Type 2 included). Clear errors when the
  record id doesn't exist ("run `hoppy billing records` to list ids") or
  the record has no `DocumentDownloadUrl` — without echoing the signed URL.
- `invoice-pdf` — **decision: no automatic fallback to the signed URL.**
  Rationale: a silent fallback would issue a hidden extra `GET /billing`
  and blur which endpoint actually failed, and `receipt-pdf` already covers
  every record type via the signed URL. Instead, improve `invoice-pdf`'s
  404 error message to say the record is likely a top-up/payment and point
  at `hoppy billing receipt-pdf`. (Feedback item 4 asked for an explicit
  choice; this is it.)

### Tests / fixtures

- `fixtures/core/billing_get.json` already contains `BillingRecords`
  (redacted, values are test contracts — do NOT touch; decision
  2026-07-10). For the receipt download test, the redacted
  `DocumentDownloadUrl` (`"<redacted>"`) is patched at test runtime to
  point at the wiremock server — fixtures stay untouched.
- `crates/bunny-net-api/tests/core/e2e/billing_api.rs`: extend the shape
  assertions to cover `billing_records` (non-empty from fixture, id > 0,
  type parses, JSON-key presence for `BillingRecords`); new test for the
  document download streaming (mock serves PDF bytes; assert **no
  `AccessKey` header** is sent on the signed-URL request).
- `crates/hoppy-cli/tests/e2e/`: CLI e2e for `billing records` (table/json,
  redaction default + `--reveal`) and `receipt-pdf` (happy path, unknown
  record id, record without `DocumentDownloadUrl`) following
  `cli_account.rs` patterns.
- Read-only API usage only; no live-API additions required (record via
  `/api-drift-sweep` later if wanted).

## Tasks

- [x] Branch `iter-83/billing-records`; set status `in-progress`
- [x] `BillingRecord` type + `billing_records` field on `BillingDetails`
      (`#[serde(default)]`, `Type` as tolerant `i32`)
- [x] Client: streaming download of a pre-signed document URL — no
      `AccessKey` header, no URL in logs/errors
- [x] CLI `hoppy billing records` (table/json/text, type names, Payer
      redaction + `--reveal`, signed URLs redacted in json)
- [x] CLI `hoppy billing receipt-pdf --record-id <ID> --output <file>`
      with clear not-found / no-document errors
- [x] `invoice-pdf`: friendlier 404 message pointing at `receipt-pdf`
- [x] API e2e tests (record shape, download streaming, AccessKey-absence
      assertion)
- [x] CLI e2e tests (records list, redaction, receipt-pdf happy/error
      paths)
- [x] Update `hoppy-knowledgebase/cli/command-tree.md` and CHANGELOG
      (Unreleased → Added)
- [x] Quality gates: `cargo fmt` → `cargo clippy --workspace --all-targets
      -- -D warnings` → `cargo test --workspace --quiet`
- [x] Dogfood against real account (`TEST_BUNNY_API_KEY`): `billing
      records`, `receipt-pdf` on a Type 2 record, `invoice-pdf` 404 message
- [ ] PR `iter-83/billing-records`, self-review diff first

## Acceptance criteria

- [x] `hoppy billing records` lists id, timestamp, amount, type (friendly
      name; raw number for unknown values), payer (redacted by default),
      invoice-available in table, json, and text formats
- [x] `hoppy billing receipt-pdf --record-id <ID> --output f.pdf` downloads
      a Type 2 top-up receipt (where `/billing/summary/{id}/pdf` 404s)
- [x] Unknown record id / missing `DocumentDownloadUrl` fail with actionable
      errors that do not contain the signed URL
- [x] No request to a `DocumentDownloadUrl` host ever carries the
      `AccessKey` header (asserted in a test)
- [x] `BillingDetails` without a `BillingRecords` key still deserializes
      (serde default)
- [x] All three quality gates pass; fixtures under `fixtures/` unchanged

## Out of scope

- Auto-fallback in `invoice-pdf` (decided against, see Design)
- `DetailedDocumentDownloadUrl` as a separate download path (covered by the
  existing `invoice-pdf` endpoint)
- Modeling `BillingHistoryChart` or other still-dropped `GET /billing`
  fields

## References

- [[iterations/iteration-75-account-billing]] — billing commands precedent
- [[decision-log]] — fixtures-as-test-contracts (2026-07-10)
- [[cli/command-tree]]
