---
title: `--record` saves live billing/payment PII to fixtures with no redaction warning
type: backlog
date: 2026-05-31
status: planned
priority: high
origin: dogfooding-2026-05-31
tags:
  - cli
  - record
  - secrets
  - pii
  - fixtures
---

# `hoppy --record <dir> auth check` leaks billing PII straight to disk

Running `--record /tmp/hoppy-rec auth check` (a single read-only command
documented in the dogfooding playbook) writes
`/tmp/hoppy-rec/core/GET_billing.json` containing the live account's:

- **Real balance** (`Balance`, `ThisMonthCharges`, `LastRechargeBalance`)
- **Payer email** in `BillingRecords[].Payer` (`james+bunnytest@ractive.ch`)
- **Payment IDs** in `BillingRecords[].PaymentId`
- **Signed invoice download URLs** with embedded tokens
  (`DocumentDownloadUrl`), e.g.
  `https://billing.b-cdn.net/.../receipt_<id>.pdf?token=<jwt-ish>&expires=…`

The dogfooding playbook tells users to run `HOPPY_RECORD_DIR=$(pwd)/fixtures
cargo test --workspace --features live-api -- --test-threads=1` and then
**commit the drift**. The "spot-check changed fixtures" step is a manual
human gate — and the URLs are signed, so silent commit-then-publish would
still leak working download links until they expire.

## Repro

```sh
rm -rf /tmp/hoppy-rec && mkdir /tmp/hoppy-rec
hoppy --record /tmp/hoppy-rec auth check
cat /tmp/hoppy-rec/core/GET_billing.json | jq '{
  Balance, ThisMonthCharges,
  PayerEmail: .BillingRecords[0].Payer,
  PaymentId: .BillingRecords[0].PaymentId,
  InvoiceUrl: .BillingRecords[0].DocumentDownloadUrl
}'
```

## Why this matters more than the existing note

[[fixture-recording-name-mismatch]] mentions "Auto-redaction of account-
specific values" as one of three deferred follow-ups. This issue is more
focused and higher-priority for two reasons:

1. The auth-check happy path — the *first* command every user runs —
   already writes a billing fixture by virtue of `auth check` calling
   `/billing` under the hood.
2. The invoice URLs are pre-signed and live; they don't need the host
   account's API key to download.

## Suggested fix

1. Recorder middleware should redact a known list of fields **before**
   writing JSON to disk:
   - `Balance`, `ThisMonthCharges`, `LastRechargeBalance` → `0`
   - `BillingRecords[].Payer` → `"redacted@example.com"`
   - `BillingRecords[].PaymentId` → `"REDACTED"`
   - `BillingRecords[].DocumentDownloadUrl`,
     `BillingRecords[].DetailedDocumentDownloadUrl` → `null`
   - any `*Email`, `*Token`, `*Password`, `*ApiKey`, `*Secret` fields
     suffixed at any depth
   - `AccountId`, `UserId`, `PayerId` → integer hash bucket or `0`
2. Expose a `--record-no-redact` escape hatch for the rare case the user
   wants verbatim recording (and is recording into a non-committed dir).
3. Update the playbook to remove the "spot-check changed fixtures" step
   (or at least demote it from "remember to do this" to "double-check
   the auto-redactor caught everything").
4. Add an integration test: record `/billing` for a known fixture
   account, assert the output JSON does not contain any of the raw
   account-id / email / payment-id values.

## Related

- [[fixture-recording-name-mismatch]] — broader recorder limitations
- [[stream-library-api-key-unrecoverable]] — adjacent secrets-handling
