use crate::auth;
use crate::cli::{
    DnsAction, DnsDnssecAction, DnsRecordAction, DnsScanAction, DnsZoneAction, OutputFormat,
};
use crate::output::{self, PaginatedListJson, TABLE_CELL_MAX, truncate_for_table};
use anyhow::{Context, Result, bail};
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{
    AddDnsRecord, ApiError, CreateDnsZone, DnsDiscoveredRecord, DnsMonitoringType, DnsRecord,
    DnsRecordScanResult, DnsRecordType, DnsSecDsRecord, DnsSmartRoutingType, DnsZone,
    LogAnonymizationType, TriggerDnsRecordScan, UpdateDnsRecord, UpdateDnsZone,
};
use std::io::{self, BufRead, Read, Write};

// ---------------------------------------------------------------------------
// Display structs — DNS Zones
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsZoneRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "Records")]
    record_count: usize,
    #[tabled(rename = "NS Detected")]
    nameservers_detected: bool,
    #[tabled(rename = "DNSSEC")]
    dns_sec_enabled: bool,
    #[tabled(rename = "Created")]
    date_created: String,
}

impl From<&DnsZone> for DnsZoneRow {
    fn from(z: &DnsZone) -> Self {
        Self {
            id: z.id,
            domain: z.domain.clone(),
            record_count: z.records.len(),
            nameservers_detected: z.nameservers_detected,
            dns_sec_enabled: z.dns_sec_enabled,
            date_created: z.date_created.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — availability
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct AvailabilityRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Available")]
    available: bool,
}

impl AvailabilityRow {
    fn new(name: &str, available: bool) -> Self {
        Self {
            name: name.to_owned(),
            available,
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — DNS Records
// ---------------------------------------------------------------------------

#[derive(Clone, serde::Serialize, tabled::Tabled)]
struct DnsRecordRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Type")]
    record_type: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "Priority")]
    priority: String,
    #[tabled(rename = "TTL")]
    ttl: i32,
    #[tabled(rename = "Disabled")]
    disabled: bool,
}

impl From<&DnsRecord> for DnsRecordRow {
    fn from(r: &DnsRecord) -> Self {
        let record_type = r
            .record_type
            .map(|t| t.to_string())
            .unwrap_or_else(|| "Unknown".to_owned());
        let priority = match r.record_type {
            Some(DnsRecordType::MX | DnsRecordType::SRV) => r.priority.to_string(),
            _ => String::new(),
        };
        Self {
            id: r.id,
            record_type,
            name: if r.name.is_empty() {
                "@".to_owned()
            } else {
                r.name.clone()
            },
            value: r.value.clone(),
            priority,
            ttl: r.ttl,
            disabled: r.disabled,
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — DNSSEC
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct DnssecRow {
    #[tabled(rename = "Enabled")]
    enabled: bool,
    #[tabled(rename = "Key Tag")]
    key_tag: i32,
    #[tabled(rename = "Algorithm")]
    algorithm: i32,
    #[tabled(rename = "Digest Type")]
    digest_type: String,
    #[tabled(rename = "Digest")]
    digest: String,
    #[tabled(rename = "Flags")]
    flags: i32,
    #[tabled(rename = "DS Configured")]
    ds_configured: bool,
}

impl From<&DnsSecDsRecord> for DnssecRow {
    fn from(r: &DnsSecDsRecord) -> Self {
        Self {
            enabled: r.enabled,
            key_tag: r.key_tag,
            algorithm: r.algorithm,
            digest_type: r.digest_type.clone().unwrap_or_else(|| "-".to_owned()),
            digest: r.digest.clone().unwrap_or_else(|| "-".to_owned()),
            flags: r.flags,
            ds_configured: r.ds_configured,
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct DnssecStatusRow {
    #[tabled(rename = "Zone ID")]
    id: i64,
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "DNSSEC Enabled")]
    enabled: bool,
}

// ---------------------------------------------------------------------------
// Display structs — DNS record scan
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsScanTriggerRow {
    #[tabled(rename = "Job ID")]
    job_id: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsScanSummaryRow {
    #[tabled(rename = "Job ID")]
    job_id: String,
    #[tabled(rename = "Zone ID")]
    zone_id: String,
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Records")]
    record_count: usize,
    #[tabled(rename = "Created")]
    created_at: String,
    #[tabled(rename = "Completed")]
    completed_at: String,
}

impl From<&DnsRecordScanResult> for DnsScanSummaryRow {
    fn from(r: &DnsRecordScanResult) -> Self {
        Self {
            job_id: r.job_id.clone().unwrap_or_else(|| "-".to_owned()),
            zone_id: r
                .zone_id
                .map(|i| i.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            domain: r.domain.clone().unwrap_or_else(|| "-".to_owned()),
            status: r
                .status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            record_count: r.records.len(),
            created_at: r.created_at.clone().unwrap_or_else(|| "-".to_owned()),
            completed_at: r.completed_at.clone().unwrap_or_else(|| "-".to_owned()),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsDiscoveredRecordRow {
    #[tabled(rename = "Type")]
    record_type: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "TTL")]
    ttl: String,
    #[tabled(rename = "Priority")]
    priority: String,
}

impl From<&DnsDiscoveredRecord> for DnsDiscoveredRecordRow {
    fn from(r: &DnsDiscoveredRecord) -> Self {
        Self {
            record_type: r
                .record_type
                .map(|t| t.to_string())
                .unwrap_or_else(|| "Unknown".to_owned()),
            name: r.name.clone().unwrap_or_else(|| "@".to_owned()),
            value: r.value.clone().unwrap_or_default(),
            ttl: r
                .ttl
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            priority: r.priority.map(|p| p.to_string()).unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &DnsAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(debug, record)?;

    match action {
        DnsAction::Zone { action } => handle_zone(&client, action, format, yes).await,
        DnsAction::Record { action } => handle_record(&client, action, format, yes).await,
    }
}

async fn handle_zone(
    client: &CoreClient,
    action: &DnsZoneAction,
    format: OutputFormat,
    yes: bool,
) -> Result<()> {
    match action {
        DnsZoneAction::List {
            search,
            page,
            per_page,
            all,
        } => {
            if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<DnsZone> = Vec::new();
                loop {
                    let result = client
                        .list_dns_zones(Some(current_page), Some(AUTO_PER_PAGE), search.as_deref())
                        .await?;
                    let has_more = result.has_more_items;
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<DnsZoneRow> =
                            result.items.iter().map(DnsZoneRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    if !has_more {
                        break;
                    }
                    current_page += 1;
                }
                if let OutputFormat::Json = format {
                    let total = accumulated.len() as i64;
                    let envelope = PaginatedListJson {
                        items: &accumulated,
                        current_page: current_page as i64,
                        total_items: total,
                        has_more_items: false,
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
            } else {
                let result = client
                    .list_dns_zones(*page, *per_page, search.as_deref())
                    .await?;
                if let OutputFormat::Json = format {
                    let envelope = PaginatedListJson {
                        items: &result.items,
                        current_page: result.current_page,
                        total_items: result.total_items,
                        has_more_items: result.has_more_items,
                    };
                    let json = serde_json::to_string_pretty(&envelope)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                } else {
                    let rows: Vec<DnsZoneRow> = result.items.iter().map(DnsZoneRow::from).collect();
                    output::print_data(&rows, format);
                }
            }
        }
        DnsZoneAction::Get { id } => {
            let zone = client.get_dns_zone(*id).await?;
            print_dns_zone(&zone, format);
        }
        DnsZoneAction::Create { domain } => {
            let body = CreateDnsZone::new(domain);
            let zone = client.create_dns_zone(&body).await?;
            print_dns_zone(&zone, format);
        }
        DnsZoneAction::Update {
            id,
            custom_nameservers_enabled,
            nameserver1,
            nameserver2,
            soa_email,
            logging_enabled,
            logging_ip_anonymization_enabled,
            log_anonymization_type,
        } => {
            if custom_nameservers_enabled.is_none()
                && nameserver1.is_none()
                && nameserver2.is_none()
                && soa_email.is_none()
                && logging_enabled.is_none()
                && logging_ip_anonymization_enabled.is_none()
                && log_anonymization_type.is_none()
            {
                bail!(
                    "at least one update flag is required (--custom-nameservers-enabled, --nameserver1, --nameserver2, --soa-email, --logging-enabled, --logging-ip-anonymization-enabled, or --log-anonymization-type)"
                );
            }
            let mut body = UpdateDnsZone::new();
            if let Some(v) = custom_nameservers_enabled {
                body = body.custom_nameservers_enabled(*v);
            }
            if let Some(v) = nameserver1 {
                body = body.nameserver1(v);
            }
            if let Some(v) = nameserver2 {
                body = body.nameserver2(v);
            }
            if let Some(v) = soa_email {
                body = body.soa_email(v);
            }
            if let Some(v) = logging_enabled {
                body = body.logging_enabled(*v);
            }
            if let Some(v) = logging_ip_anonymization_enabled {
                body = body.logging_ip_anonymization_enabled(*v);
            }
            if let Some(v) = log_anonymization_type {
                let parsed: LogAnonymizationType = v.parse().map_err(anyhow::Error::msg)?;
                body = body.log_anonymization_type(parsed);
            }
            let zone = client.update_dns_zone(*id, &body).await?;
            print_dns_zone(&zone, format);
        }
        DnsZoneAction::Check { domain } => {
            let availability = client.check_dns_zone_availability(domain).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&availability)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                output::print_single(
                    &AvailabilityRow::new(domain, availability.available),
                    format,
                );
            }
        }
        DnsZoneAction::Delete { id } => {
            if !yes {
                eprint!("Delete DNS zone {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_dns_zone(*id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "dns-zone",
                serde_json::json!({ "Id": id }),
                &format!("Deleted DNS zone {id}"),
            );
        }
        DnsZoneAction::Export { id } => {
            let raw = client.export_dns_zone(*id).await?;
            let content = if raw.trim().is_empty() {
                let zone = client.get_dns_zone(*id).await?;
                format!(";; zone {} — 0 records\n", zone.domain)
            } else if !raw.ends_with('\n') {
                format!("{raw}\n")
            } else {
                raw
            };
            match format {
                OutputFormat::Json => {
                    let json =
                        serde_json::to_string_pretty(&serde_json::json!({ "Bind": content }))
                            .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
                OutputFormat::Table | OutputFormat::Text => {
                    print!("{content}");
                }
            }
        }
        DnsZoneAction::Import { id, file } => {
            let zone_data = match file {
                Some(path) => std::fs::read_to_string(path)
                    .with_context(|| format!("failed to read zone file: {path}"))?,
                None => {
                    let mut buf = String::new();
                    io::stdin()
                        .lock()
                        .read_to_string(&mut buf)
                        .context("failed to read zone file from stdin")?;
                    buf
                }
            };
            let result = client.import_dns_zone(*id, &zone_data).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                eprintln!(
                    "Import complete: {} successful, {} failed, {} skipped",
                    result.records_successful, result.records_failed, result.records_skipped
                );
            }
        }
        DnsZoneAction::Dnssec { action } => {
            handle_dnssec(client, action, format, yes).await?;
            return Ok(());
        }
        DnsZoneAction::IssueCert { id } => {
            if let Err(err) = client.issue_dns_zone_wildcard_certificate(*id).await {
                return Err(annotate_issue_cert_error(err, *id));
            }
            output::print_mutation_result(
                format,
                "issue-wildcard-cert",
                "dns-zone",
                serde_json::json!({ "Id": id }),
                &format!("Issued wildcard certificate for DNS zone {id}"),
            );
            return Ok(());
        }
        DnsZoneAction::Scan { action } => {
            handle_scan(client, action, format).await?;
            return Ok(());
        }
        DnsZoneAction::Statistics {
            id,
            date_from,
            date_to,
        } => {
            let stats = client
                .get_dns_zone_statistics(*id, date_from.as_deref(), date_to.as_deref())
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                #[derive(serde::Serialize, tabled::Tabled)]
                struct Row {
                    #[tabled(rename = "Metric")]
                    metric: String,
                    #[tabled(rename = "Value")]
                    value: String,
                }
                let rows = vec![Row {
                    metric: "Total Queries Served".to_string(),
                    value: stats.total_queries_served.to_string(),
                }];
                output::print_data(&rows, format);
            }
        }
    }
    Ok(())
}

/// Append a delegation hint when `issue-cert` returns a generic upstream 500.
///
/// The bunny.net API responds with a structureless 500 when the zone is not
/// delegated to bunny nameservers (the DNS-01 challenge can't complete). The
/// raw message is "An error has occurred." — we wrap it as `anyhow` context so
/// the original `ApiError` is preserved as `source()` for debug printing while
/// the user-facing message gains an actionable next step.
fn annotate_issue_cert_error(err: anyhow::Error, zone_id: i64) -> anyhow::Error {
    if let Some(api_err) = err.downcast_ref::<ApiError>()
        && api_err.status_code == 500
        && api_err.error_key.is_empty()
    {
        return err.context(format!(
            "hint: the zone must be delegated to bunny.net nameservers before a certificate can be issued. Set NS records to the values from `hoppy dns zone get --id {zone_id}`."
        ));
    }
    err
}

async fn handle_dnssec(
    client: &CoreClient,
    action: &DnsDnssecAction,
    format: OutputFormat,
    yes: bool,
) -> Result<()> {
    match action {
        DnsDnssecAction::Enable { id } => {
            let ds = client.enable_dns_zone_dnssec(*id).await?;
            print_dnssec(&ds, format)?;
        }
        DnsDnssecAction::Disable { id } => {
            if !yes {
                eprintln!(
                    "WARNING: disabling DNSSEC at bunny.net while DS records remain at your registrar will break DNS resolution. Remove the DS records from your registrar first."
                );
                eprint!("Disable DNSSEC for DNS zone {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let ds = client.disable_dns_zone_dnssec(*id).await?;
            print_dnssec(&ds, format)?;
        }
        DnsDnssecAction::Status { id } => {
            let zone = client.get_dns_zone(*id).await?;
            if let OutputFormat::Json = format {
                // Enrich with DS record details when DNSSEC is enabled.
                // enable_dns_zone_dnssec is idempotent — calling it on an
                // already-enabled zone returns the DS record without changing
                // anything.
                if zone.dns_sec_enabled {
                    let ds = client.enable_dns_zone_dnssec(*id).await?;
                    let row = serde_json::json!({
                        "Id": zone.id,
                        "Domain": zone.domain,
                        "DnsSecEnabled": zone.dns_sec_enabled,
                        "DsRecord": ds.ds_record,
                        "Digest": ds.digest,
                        "DigestType": ds.digest_type,
                        "Algorithm": ds.algorithm,
                        "KeyTag": ds.key_tag,
                        "Flags": ds.flags,
                        "DsConfigured": ds.ds_configured,
                    });
                    let json = serde_json::to_string_pretty(&row)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                } else {
                    let row = serde_json::json!({
                        "Id": zone.id,
                        "Domain": zone.domain,
                        "DnsSecEnabled": zone.dns_sec_enabled,
                    });
                    let json = serde_json::to_string_pretty(&row)
                        .context("failed to serialize to JSON")?;
                    println!("{json}");
                }
            } else if zone.dns_sec_enabled {
                // Fetch DS record details (idempotent enable call).
                let ds = client.enable_dns_zone_dnssec(*id).await?;
                let row = DnssecRow::from(&ds);
                // Show zone-level fields as a header, then the DS record row.
                eprintln!(
                    "Zone ID: {}  Domain: {}  DNSSEC: enabled",
                    zone.id, zone.domain
                );
                output::print_single(&row, format);
                if let Some(rec) = &ds.ds_record {
                    eprintln!();
                    eprintln!("DS record (copy this to your domain registrar):");
                    eprintln!("  {rec}");
                }
            } else {
                let row = DnssecStatusRow {
                    id: zone.id,
                    domain: zone.domain.clone(),
                    enabled: zone.dns_sec_enabled,
                };
                output::print_single(&row, format);
            }
        }
    }
    Ok(())
}

async fn handle_scan(
    client: &CoreClient,
    action: &DnsScanAction,
    format: OutputFormat,
) -> Result<()> {
    match action {
        DnsScanAction::Start { id, domain } => {
            let body = match (id, domain) {
                (Some(zone_id), None) => TriggerDnsRecordScan::for_zone(*zone_id),
                (None, Some(d)) => TriggerDnsRecordScan::for_domain(d),
                (None, None) => unreachable!("clap ArgGroup ensures one of --id/--domain is set"),
                (Some(_), Some(_)) => {
                    unreachable!(
                        "clap conflicts_with ensures --id and --domain are mutually exclusive"
                    )
                }
            };
            let trigger = client.trigger_dns_record_scan(&body).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&trigger)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let row = DnsScanTriggerRow {
                    job_id: trigger.job_id.clone().unwrap_or_else(|| "-".to_owned()),
                    status: trigger
                        .status
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "-".to_owned()),
                };
                output::print_single(&row, format);
                let next_cmd = match (id, domain) {
                    (Some(zone_id), _) => format!("hoppy dns zone scan results --id {zone_id}"),
                    (None, Some(d)) => format!("hoppy dns zone scan results --domain {d}"),
                    (None, None) => unreachable!(),
                };
                output::hints::tip(&format!("Run: {next_cmd}"));
            }
        }
        DnsScanAction::Results { id, domain } => {
            let (zone_id, resolved_domain) = match (id, domain) {
                (Some(zid), None) => (*zid, None),
                (None, Some(d)) => (resolve_domain_to_zone_id(client, d).await?, Some(d.clone())),
                (None, None) => unreachable!("clap ArgGroup ensures one of --id/--domain is set"),
                (Some(_), Some(_)) => unreachable!(
                    "clap conflicts_with ensures --id and --domain are mutually exclusive"
                ),
            };
            let mut result = client.get_dns_zone_record_scan(zone_id).await?;
            if result.domain.is_none() {
                result.domain = match resolved_domain {
                    Some(d) => Some(d),
                    None => Some(client.get_dns_zone(zone_id).await?.domain),
                };
            }
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let summary = DnsScanSummaryRow::from(&result);
                output::print_single(&summary, format);
                if !result.records.is_empty() {
                    let rows: Vec<DnsDiscoveredRecordRow> = result
                        .records
                        .iter()
                        .map(DnsDiscoveredRecordRow::from)
                        .collect();
                    output::print_data(&rows, format);
                }
                if let Some(err) = &result.error {
                    eprintln!("Scan error: {err}");
                }
            }
        }
    }
    Ok(())
}

/// Resolve a domain name to a DNS zone id by searching the zone list.
///
/// The bunny.net API only exposes scan results keyed by zone id, so callers
/// passing `--domain` need a zone to exist. We do an exact (case-insensitive)
/// match on the `Domain` field; pure-prefix matches from the API search are
/// rejected to avoid silently picking the wrong zone.
async fn resolve_domain_to_zone_id(client: &CoreClient, domain: &str) -> Result<i64> {
    let needle = domain.trim().to_ascii_lowercase();
    let page = client
        .list_dns_zones(None, None, Some(&needle))
        .await
        .with_context(|| format!("failed to look up DNS zone for domain '{needle}'"))?;
    let exact = page
        .items
        .iter()
        .find(|z| z.domain.eq_ignore_ascii_case(&needle));
    match exact {
        Some(z) => Ok(z.id),
        None => bail!(
            "no DNS zone found for domain '{needle}' — the bunny.net API only \
             exposes scan results by zone id, so the zone must exist first. \
             Create it with `hoppy dns zone create --domain {needle}` and then \
             re-run `hoppy dns zone scan results --domain {needle}`."
        ),
    }
}

fn print_dnssec(ds: &DnsSecDsRecord, format: OutputFormat) -> Result<()> {
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(ds).context("failed to serialize to JSON")?;
        println!("{json}");
    } else {
        let row = DnssecRow::from(ds);
        output::print_single(&row, format);
        if let Some(rec) = &ds.ds_record {
            eprintln!();
            eprintln!("DS record (copy this to your domain registrar):");
            eprintln!("  {rec}");
        }
    }
    Ok(())
}

async fn handle_record(
    client: &CoreClient,
    action: &DnsRecordAction,
    format: OutputFormat,
    yes: bool,
) -> Result<()> {
    match action {
        DnsRecordAction::List {
            zone_id,
            page,
            per_page,
            all,
        } => {
            // Backed by the dedicated GET /dnszone/{id}/records endpoint.
            let records = if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<DnsRecord> = Vec::new();
                loop {
                    let result = client
                        .list_dns_zone_records(*zone_id, Some(current_page), Some(AUTO_PER_PAGE))
                        .await?;
                    let has_more = result.has_more_items;
                    accumulated.extend(result.items);
                    if !has_more {
                        break;
                    }
                    current_page += 1;
                }
                accumulated
            } else {
                client
                    .list_dns_zone_records(*zone_id, *page, *per_page)
                    .await?
                    .items
            };
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&records)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let rows: Vec<DnsRecordRow> = records.iter().map(DnsRecordRow::from).collect();
                if let OutputFormat::Table = format {
                    let mut truncated_rows = rows.clone();
                    let mut any_truncated = false;
                    for row in &mut truncated_rows {
                        let (v, t) = truncate_for_table(&row.value, TABLE_CELL_MAX);
                        row.value = v;
                        any_truncated |= t;
                    }
                    output::print_data(&truncated_rows, format);
                    if any_truncated {
                        output::hints::tip(
                            "some Value fields were truncated — use --format json for full values",
                        );
                    }
                } else {
                    output::print_data(&rows, format);
                }
            }
        }
        DnsRecordAction::Add {
            zone_id,
            r#type,
            name,
            value,
            ttl,
            priority,
            weight,
            port,
            flags,
            tag,
            pull_zone_id,
            script_id,
            accelerated,
            smart_routing_type,
            monitor_type,
            geolocation_latitude,
            geolocation_longitude,
            latency_zone,
            auto_ssl_issuance,
            disabled,
            comment,
        } => {
            let record_type: DnsRecordType = r#type.parse()?;
            let mut body = AddDnsRecord::new(record_type, value);
            if let Some(n) = name {
                body = body.name(n);
            }
            if let Some(t) = ttl {
                body = body.ttl(*t);
            }
            if let Some(p) = priority {
                body = body.priority(*p);
            }
            if let Some(w) = weight {
                body = body.weight(*w);
            }
            if let Some(p) = port {
                body = body.port(*p);
            }
            if let Some(f) = flags {
                body = body.flags(*f);
            }
            if let Some(t) = tag {
                body = body.tag(t);
            }
            if let Some(id) = pull_zone_id {
                body = body.pull_zone_id(*id);
            }
            if let Some(id) = script_id {
                body = body.script_id(*id);
            }
            if let Some(a) = accelerated {
                body = body.accelerated(*a);
            }
            if let Some(s) = smart_routing_type {
                let parsed: DnsSmartRoutingType = s.parse().map_err(anyhow::Error::msg)?;
                body = body.smart_routing_type(parsed);
            }
            if let Some(m) = monitor_type {
                let parsed: DnsMonitoringType = m.parse().map_err(anyhow::Error::msg)?;
                body = body.monitor_type(parsed);
            }
            if let Some(lat) = geolocation_latitude {
                body = body.geolocation_latitude(*lat);
            }
            if let Some(lon) = geolocation_longitude {
                body = body.geolocation_longitude(*lon);
            }
            if let Some(z) = latency_zone {
                body = body.latency_zone(z);
            }
            if let Some(a) = auto_ssl_issuance {
                body = body.auto_ssl_issuance(*a);
            }
            if let Some(d) = disabled {
                body = body.disabled(*d);
            }
            if let Some(c) = comment {
                body = body.comment(c);
            }
            let record = client.add_dns_record(*zone_id, &body).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&record).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let row = DnsRecordRow::from(&record);
                output::print_single(&row, format);
            }
        }
        DnsRecordAction::Update {
            zone_id,
            record_id,
            r#type,
            value,
            name,
            ttl,
            priority,
            weight,
            port,
            flags,
            tag,
            pull_zone_id,
            script_id,
            accelerated,
            smart_routing_type,
            monitor_type,
            geolocation_latitude,
            geolocation_longitude,
            latency_zone,
            auto_ssl_issuance,
            disabled,
            comment,
        } => {
            // Read-modify-write: bunny's record update is not a partial PATCH —
            // omitted fields are treated as cleared. Fetch the current record
            // (records are embedded in the zone response) so unspecified flags
            // keep their existing value and SRV/CAA fields survive the round-trip.
            let zone = client.get_dns_zone(*zone_id).await?;
            let current = zone
                .records
                .iter()
                .find(|r| r.id == *record_id)
                .with_context(|| format!("DNS record {record_id} not found in zone {zone_id}"))?;

            let record_type: DnsRecordType = match r#type {
                Some(t) => t.parse()?,
                None => current.record_type.with_context(|| {
                    format!("DNS record {record_id} has no type; re-specify it with --type")
                })?,
            };
            let value = value.clone().unwrap_or_else(|| current.value.clone());
            let mut body = UpdateDnsRecord::new(*record_id, record_type, value);

            // Name: apply override, else carry the current value (skip empty apex).
            match name {
                Some(n) => body = body.name(n),
                None if !current.name.is_empty() => body = body.name(&current.name),
                None => {}
            }
            body = body.ttl(ttl.unwrap_or(current.ttl));
            body = body.priority(priority.unwrap_or(current.priority));
            body = body.weight(weight.unwrap_or(current.weight));
            body = body.port(port.unwrap_or(current.port));
            body = body.flags(flags.unwrap_or(current.flags));
            match tag {
                Some(t) => body = body.tag(t),
                None => {
                    if let Some(t) = &current.tag {
                        body = body.tag(t);
                    }
                }
            }
            body = body.disabled(disabled.unwrap_or(current.disabled));
            match comment {
                Some(c) => body = body.comment(c),
                None => {
                    if let Some(c) = &current.comment {
                        body = body.comment(c);
                    }
                }
            }

            // Linked / smart-routing fields: apply the override when supplied,
            // otherwise carry the record's existing value so the round-trip is
            // non-lossy (bunny treats omitted fields as cleared).
            match (pull_zone_id, current.pull_zone_id) {
                (Some(id), _) => body = body.pull_zone_id(*id),
                (None, cur) if cur != 0 => body = body.pull_zone_id(cur),
                (None, _) => {}
            }
            match (script_id, current.script_id) {
                (Some(id), _) => body = body.script_id(*id),
                (None, cur) if cur != 0 => body = body.script_id(cur),
                (None, _) => {}
            }
            body = body.accelerated(accelerated.unwrap_or(current.accelerated));
            match smart_routing_type {
                Some(s) => {
                    let parsed: DnsSmartRoutingType = s.parse().map_err(anyhow::Error::msg)?;
                    body = body.smart_routing_type(parsed);
                }
                None => {
                    if let Some(s) = current.smart_routing_type {
                        body = body.smart_routing_type(s);
                    }
                }
            }
            match monitor_type {
                Some(m) => {
                    let parsed: DnsMonitoringType = m.parse().map_err(anyhow::Error::msg)?;
                    body = body.monitor_type(parsed);
                }
                None => {
                    if let Some(m) = current.monitor_type {
                        body = body.monitor_type(m);
                    }
                }
            }
            body = body
                .geolocation_latitude(geolocation_latitude.unwrap_or(current.geolocation_latitude));
            body = body.geolocation_longitude(
                geolocation_longitude.unwrap_or(current.geolocation_longitude),
            );
            match latency_zone {
                Some(z) => body = body.latency_zone(z),
                None => {
                    if let Some(z) = &current.latency_zone {
                        body = body.latency_zone(z);
                    }
                }
            }
            body = body.auto_ssl_issuance(auto_ssl_issuance.unwrap_or(current.auto_ssl_issuance));

            client
                .update_dns_record(*zone_id, *record_id, &body)
                .await?;
            output::print_mutation_result(
                format,
                "update",
                "dns-record",
                serde_json::json!({ "ZoneId": zone_id, "Id": record_id }),
                &format!("Updated DNS record {record_id} in zone {zone_id}"),
            );
        }
        DnsRecordAction::Delete { zone_id, record_id } => {
            if !yes {
                eprint!("Delete DNS record {record_id} from zone {zone_id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_dns_record(*zone_id, *record_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "dns-record",
                serde_json::json!({}),
                &format!("Deleted DNS record {record_id} from zone {zone_id}"),
            );
        }
    }
    Ok(())
}

fn print_dns_zone(zone: &DnsZone, format: OutputFormat) {
    let redact_cfg = crate::redact::RedactConfig::default();
    let cmd = format!("dns zone get --id {}", zone.id);
    output::print_single_vertical_with_cmd(zone, format, &redact_cfg, Some(&cmd));
}
