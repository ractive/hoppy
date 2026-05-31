use crate::auth;
use crate::cli::{
    DnsAction, DnsDnssecAction, DnsRecordAction, DnsScanAction, DnsZoneAction, OutputFormat,
};
use crate::output::{self, PaginatedListJson, TABLE_CELL_MAX, truncate_for_table};
use anyhow::{Context, Result, bail};
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{
    AddDnsRecord, CreateDnsZone, DnsDiscoveredRecord, DnsRecord, DnsRecordScanResult,
    DnsRecordType, DnsSecDsRecord, DnsZone, TriggerDnsRecordScan, UpdateDnsRecord, UpdateDnsZone,
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
        } => {
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
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<DnsZoneRow> = result.items.iter().map(DnsZoneRow::from).collect();
                output::print_data(&rows, format);
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
        } => {
            if custom_nameservers_enabled.is_none()
                && nameserver1.is_none()
                && nameserver2.is_none()
                && soa_email.is_none()
                && logging_enabled.is_none()
                && logging_ip_anonymization_enabled.is_none()
            {
                bail!(
                    "at least one update flag is required (--custom-nameservers-enabled, --nameserver1, --nameserver2, --soa-email, --logging-enabled, or --logging-ip-anonymization-enabled)"
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
            let zone = client.update_dns_zone(*id, &body).await?;
            print_dns_zone(&zone, format);
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
            let content = client.export_dns_zone(*id).await?;
            print!("{content}");
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
            client.issue_dns_zone_wildcard_certificate(*id).await?;
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
            }
        }
        DnsScanAction::Results { id } => {
            let result = client.get_dns_zone_record_scan(*id).await?;
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
        DnsRecordAction::List { zone_id } => {
            // Records are embedded in the zone response
            let zone = client.get_dns_zone(*zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&zone.records)
                    .expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<DnsRecordRow> = zone.records.iter().map(DnsRecordRow::from).collect();
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
            comment,
        } => {
            let record_type: DnsRecordType = r#type.parse()?;
            let mut body = UpdateDnsRecord::new(*record_id, record_type, value);
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
            if let Some(c) = comment {
                body = body.comment(c);
            }
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
