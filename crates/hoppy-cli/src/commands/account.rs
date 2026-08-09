//! Account / admin commands (iter-75): API keys, billing summary and
//! payment requests, invoice PDF downloads, region/country reference data,
//! global search, and the user audit log.

use crate::auth;
use crate::cli::{
    ApikeyAction, BillingAction, CountryAction, OutputFormat, RegionAction, UserAction,
};
use crate::output;
use crate::redact::placeholder;
use anyhow::{Context, Result};
use bunny_net_api::core::types::{
    ApiKey, BillingSummaryEntry, Country, PaymentRequest, Region, SearchResults, UserAuditLog,
    UserAuditLogList, UserAuditQuery,
};

// ---------------------------------------------------------------------------
// Display rows
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ApiKeyRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Key")]
    key: String,
    #[tabled(rename = "Roles")]
    roles: String,
}

impl ApiKeyRow {
    fn from_key(k: &ApiKey, reveal: bool) -> Self {
        let raw = k.key.clone().unwrap_or_default();
        let key = if reveal { raw } else { placeholder(&raw) };
        let roles = k.roles.as_ref().map(|r| r.join(", ")).unwrap_or_default();
        Self {
            id: k.id,
            key,
            roles,
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct BillingSummaryRow {
    #[tabled(rename = "Pull Zone ID")]
    pull_zone_id: i64,
    #[tabled(rename = "Monthly Usage")]
    monthly_usage: String,
    #[tabled(rename = "Bandwidth")]
    bandwidth: String,
}

impl From<&BillingSummaryEntry> for BillingSummaryRow {
    fn from(e: &BillingSummaryEntry) -> Self {
        Self {
            pull_zone_id: e.pull_zone_id,
            monthly_usage: format!("${:.4}", e.monthly_usage),
            bandwidth: format_bytes(e.monthly_bandwidth_used),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct PaymentRequestRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Amount")]
    amount: String,
    #[tabled(rename = "Due")]
    date_due: String,
    #[tabled(rename = "Paid")]
    paid: bool,
    #[tabled(rename = "Description")]
    description: String,
}

impl From<&PaymentRequest> for PaymentRequestRow {
    fn from(p: &PaymentRequest) -> Self {
        Self {
            id: p.id,
            amount: format!("${:.2}", p.amount),
            date_due: p.date_due.clone().unwrap_or_default(),
            paid: p.paid,
            description: p.description.clone().unwrap_or_default(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct RegionRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Code")]
    code: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Continent")]
    continent: String,
    #[tabled(rename = "Country")]
    country: String,
    #[tabled(rename = "$/GB")]
    price: String,
}

impl From<&Region> for RegionRow {
    fn from(r: &Region) -> Self {
        Self {
            id: r.id,
            code: r.region_code.clone().unwrap_or_default(),
            name: r.name.clone().unwrap_or_default(),
            continent: r.continent_code.clone().unwrap_or_default(),
            country: r.country_code.clone().unwrap_or_default(),
            price: format!("${:.4}", r.price_per_gigabyte),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct CountryRow {
    #[tabled(rename = "ISO")]
    iso_code: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "EU")]
    is_eu: bool,
    #[tabled(rename = "Tax Rate")]
    tax_rate: String,
}

impl From<&Country> for CountryRow {
    fn from(c: &Country) -> Self {
        Self {
            iso_code: c.iso_code.clone().unwrap_or_default(),
            name: c.name.clone().unwrap_or_default(),
            is_eu: c.is_eu,
            tax_rate: format!("{:.1}%", c.tax_rate * 100.0),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct SearchResultRow {
    #[tabled(rename = "Type")]
    result_type: String,
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
}

#[derive(serde::Serialize, tabled::Tabled)]
struct AuditLogRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Product")]
    product: String,
    #[tabled(rename = "Action")]
    action: String,
    #[tabled(rename = "Resource Type")]
    resource_type: String,
    #[tabled(rename = "Resource ID")]
    resource_id: String,
    #[tabled(rename = "Actor")]
    actor_id: String,
}

impl From<&UserAuditLog> for AuditLogRow {
    fn from(l: &UserAuditLog) -> Self {
        Self {
            timestamp: l.timestamp.clone().unwrap_or_default(),
            product: l.product.clone().unwrap_or_default(),
            action: l.action.clone().unwrap_or_default(),
            resource_type: l.resource_type.clone().unwrap_or_default(),
            resource_id: l.resource_id.clone().unwrap_or_default(),
            actor_id: l.actor_id.clone().unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn handle_apikey(
    action: &ApikeyAction,
    format: OutputFormat,
    debug: bool,
    reveal_global: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        ApikeyAction::List {
            page,
            per_page,
            reveal,
        } => {
            let client = auth::core_client_with_reveal(debug, record, *reveal || reveal_global)?;
            let list = client.list_api_keys(*page, *per_page).await?;
            let reveal = *reveal || reveal_global;

            match format {
                OutputFormat::Json => {
                    // Redact key values in JSON unless revealed so a piped
                    // `--format json` doesn't leak secrets into a logfile.
                    let mut value =
                        serde_json::to_value(&list).context("failed to serialize to JSON")?;
                    if !reveal {
                        redact_api_keys_json(&mut value);
                    }
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&value)
                            .context("failed to serialize to JSON")?
                    );
                }
                _ => {
                    let rows: Vec<ApiKeyRow> = list
                        .items
                        .iter()
                        .map(|k| ApiKeyRow::from_key(k, reveal))
                        .collect();
                    output::print_data(&rows, format);
                    if !reveal {
                        output::hints::tip("re-run with --reveal to show full key values");
                    }
                }
            }
        }
    }
    Ok(())
}

pub async fn handle_billing(
    action: &BillingAction,
    format: OutputFormat,
    debug: bool,
    quiet: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(debug, record)?;
    match action {
        BillingAction::Summary => {
            let entries = client.get_billing_summary().await?;
            if let OutputFormat::Json = format {
                print_json(&entries)?;
            } else {
                let rows: Vec<BillingSummaryRow> =
                    entries.iter().map(BillingSummaryRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        BillingAction::PaymentRequests => {
            let requests = client.list_payment_requests().await?;
            if let OutputFormat::Json = format {
                print_json(&requests)?;
            } else {
                let rows: Vec<PaymentRequestRow> =
                    requests.iter().map(PaymentRequestRow::from).collect();
                output::print_data(&rows, format);
                output::hints::tip(
                    "download an invoice: hoppy billing payment-request-pdf --id <ID> --output invoice.pdf",
                );
            }
        }
        BillingAction::InvoicePdf { record_id, output } => {
            let mut file = std::fs::File::create(output)
                .with_context(|| format!("creating output file: {output}"))?;
            let n = client
                .download_billing_invoice_pdf(*record_id, &mut file)
                .await?;
            report_pdf_saved(format, n, output, quiet);
        }
        BillingAction::PaymentRequestPdf { id, output } => {
            let mut file = std::fs::File::create(output)
                .with_context(|| format!("creating output file: {output}"))?;
            let n = client.download_payment_request_pdf(*id, &mut file).await?;
            report_pdf_saved(format, n, output, quiet);
        }
    }
    Ok(())
}

pub async fn handle_region(
    action: &RegionAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        RegionAction::List => {
            let client = auth::core_client(debug, record)?;
            let regions = client.list_regions().await?;
            if let OutputFormat::Json = format {
                print_json(&regions)?;
            } else {
                let rows: Vec<RegionRow> = regions.iter().map(RegionRow::from).collect();
                output::print_data(&rows, format);
            }
        }
    }
    Ok(())
}

pub async fn handle_country(
    action: &CountryAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        CountryAction::List => {
            let client = auth::core_client(debug, record)?;
            let countries = client.list_countries().await?;
            if let OutputFormat::Json = format {
                print_json(&countries)?;
            } else {
                let rows: Vec<CountryRow> = countries.iter().map(CountryRow::from).collect();
                output::print_data(&rows, format);
                output::hints::tip(
                    "use an ISO code with: hoppy pull-zone update --id <id> --blocked-countries <CODE>",
                );
            }
        }
    }
    Ok(())
}

pub async fn handle_search(
    query: &str,
    from: Option<i32>,
    size: Option<i32>,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(debug, record)?;
    let results: SearchResults = client.search(query, from, size).await?;
    if let OutputFormat::Json = format {
        print_json(&results)?;
    } else {
        let rows: Vec<SearchResultRow> = results
            .search_results
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|item| SearchResultRow {
                result_type: item.result_type.clone().unwrap_or_default(),
                id: item.id,
                name: item.name.clone().unwrap_or_default(),
            })
            .collect();
        output::print_data(&rows, format);
        if results.total > results.from + results.size {
            output::hints::tip(&format!(
                "more results available — re-run with --from {}",
                results.from + results.size
            ));
        }
    }
    Ok(())
}

pub async fn handle_user(
    action: &UserAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        UserAction::Audit {
            date,
            product,
            resource_type,
            resource_id,
            actor_id,
            order,
            continuation_token,
            limit,
        } => {
            let client = auth::core_client(debug, record)?;
            let query = UserAuditQuery {
                product: product.clone(),
                resource_type: resource_type.clone(),
                resource_id: resource_id.clone(),
                actor_id: actor_id.clone(),
                order: order.map(Into::into),
                continuation_token: continuation_token.clone(),
                limit: *limit,
            };
            let log: UserAuditLogList = client.get_user_audit(date, &query).await?;
            if let OutputFormat::Json = format {
                print_json(&log)?;
            } else {
                let rows: Vec<AuditLogRow> = log
                    .logs
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(AuditLogRow::from)
                    .collect();
                output::print_data(&rows, format);
                if log.has_more_data
                    && let Some(token) = &log.continuation_token
                {
                    output::hints::tip(&format!(
                        "more entries — re-run with --continuation-token {token}"
                    ));
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("failed to serialize to JSON")?
    );
    Ok(())
}

/// Emit a success message / envelope after a PDF download.
fn report_pdf_saved(format: OutputFormat, bytes: u64, path: &str, quiet: bool) {
    match format {
        OutputFormat::Json => {
            output::print_mutation_result(
                format,
                "download",
                "invoice-pdf",
                serde_json::json!({ "Path": path, "Bytes": bytes }),
                "",
            );
        }
        _ => {
            if !quiet {
                eprintln!("Saved {bytes} bytes to {path}");
            }
        }
    }
}

/// Redact `Key` fields inside an `/apikey` list JSON payload.
fn redact_api_keys_json(value: &mut serde_json::Value) {
    if let Some(items) = value.get_mut("Items").and_then(|v| v.as_array_mut()) {
        for item in items {
            if let Some(key) = item.get_mut("Key") {
                let raw = key.as_str().unwrap_or("");
                *key = serde_json::Value::String(placeholder(raw));
            }
        }
    }
}

fn format_bytes(bytes: i64) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;
    const KB: f64 = 1_024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn apikey_row_redacts_by_default() {
        let k = ApiKey {
            id: 7,
            key: Some("supersecretvalue".to_owned()),
            roles: Some(vec!["Admin".to_owned(), "Billing".to_owned()]),
        };
        let redacted = ApiKeyRow::from_key(&k, false);
        assert_eq!(redacted.key, "<set, length=16>");
        assert_eq!(redacted.roles, "Admin, Billing");
        let revealed = ApiKeyRow::from_key(&k, true);
        assert_eq!(revealed.key, "supersecretvalue");
    }

    #[test]
    fn apikey_row_handles_missing_key() {
        let k = ApiKey {
            id: 1,
            key: None,
            roles: None,
        };
        let row = ApiKeyRow::from_key(&k, false);
        assert_eq!(row.key, "<unset>");
        assert_eq!(row.roles, "");
    }

    #[test]
    fn redact_api_keys_json_masks_values() {
        let mut v = json!({
            "Items": [
                {"Id": 1, "Key": "abcdef", "Roles": ["Admin"]},
                {"Id": 2, "Key": "", "Roles": []},
            ],
            "CurrentPage": 1,
            "TotalItems": 2,
            "HasMoreItems": false,
        });
        redact_api_keys_json(&mut v);
        assert_eq!(v["Items"][0]["Key"], json!("<set, length=6>"));
        assert_eq!(v["Items"][1]["Key"], json!("<unset>"));
        // Non-secret fields untouched.
        assert_eq!(v["Items"][0]["Id"], json!(1));
        assert_eq!(v["TotalItems"], json!(2));
    }

    #[test]
    fn billing_summary_row_formats() {
        let e = BillingSummaryEntry {
            pull_zone_id: 42,
            monthly_usage: 1.2345,
            monthly_bandwidth_used: 2_147_483_648,
        };
        let row = BillingSummaryRow::from(&e);
        assert_eq!(row.pull_zone_id, 42);
        assert_eq!(row.monthly_usage, "$1.2345");
        assert_eq!(row.bandwidth, "2.00 GB");
    }

    #[test]
    fn payment_request_row_formats() {
        let p = PaymentRequest {
            id: 9,
            amount: 12.5,
            date_generated: None,
            date_due: Some("2026-08-01T00:00:00Z".to_owned()),
            description: Some("Monthly usage".to_owned()),
            paid: false,
            date_paid: None,
            billing_invoice_id: None,
            billing_invoice_download_link: None,
            bank_transfer_reference: None,
            tax_rate: 0.0,
            taxed_amount: 0.0,
        };
        let row = PaymentRequestRow::from(&p);
        assert_eq!(row.amount, "$12.50");
        assert_eq!(row.date_due, "2026-08-01T00:00:00Z");
        assert!(!row.paid);
        assert_eq!(row.description, "Monthly usage");
    }

    #[test]
    fn region_row_formats() {
        let r = Region {
            id: 3,
            name: Some("Frankfurt".to_owned()),
            price_per_gigabyte: 0.01,
            region_code: Some("DE".to_owned()),
            continent_code: Some("EU".to_owned()),
            country_code: Some("DE".to_owned()),
            latitude: 50.0,
            longitude: 8.0,
            allow_latency_routing: true,
        };
        let row = RegionRow::from(&r);
        assert_eq!(row.code, "DE");
        assert_eq!(row.continent, "EU");
        assert_eq!(row.price, "$0.0100");
    }

    #[test]
    fn country_row_formats_tax_rate() {
        let c = Country {
            name: Some("Germany".to_owned()),
            iso_code: Some("DE".to_owned()),
            is_eu: true,
            tax_rate: 0.19,
            tax_prefix: None,
            flag_url: None,
            pop_list: None,
        };
        let row = CountryRow::from(&c);
        assert_eq!(row.iso_code, "DE");
        assert!(row.is_eu);
        assert_eq!(row.tax_rate, "19.0%");
    }

    #[test]
    fn audit_log_row_maps_fields() {
        let l = UserAuditLog {
            timestamp: Some("2026-07-01T10:00:00Z".to_owned()),
            product: Some("CDN".to_owned()),
            resource_type: Some("PullZone".to_owned()),
            resource_id: Some("12345".to_owned()),
            resource_owner: None,
            action: Some("Update".to_owned()),
            actor_id: Some("user-1".to_owned()),
            actor_type: Some("User".to_owned()),
            diff: None,
        };
        let row = AuditLogRow::from(&l);
        assert_eq!(row.product, "CDN");
        assert_eq!(row.action, "Update");
        assert_eq!(row.resource_type, "PullZone");
        assert_eq!(row.resource_id, "12345");
        assert_eq!(row.actor_id, "user-1");
    }

    #[test]
    fn format_bytes_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }
}
