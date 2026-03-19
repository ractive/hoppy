use crate::auth;
use crate::cli::{DnsAction, DnsRecordAction, DnsZoneAction, OutputFormat};
use crate::output::{self, PaginatedListJson};
use anyhow::{Result, bail};
use bunny_api_core::CoreClient;
use bunny_api_core::types::{
    AddDnsRecord, CreateDnsZone, DnsRecord, DnsRecordType, DnsZone, UpdateDnsRecord, UpdateDnsZone,
};
use std::io::{self, BufRead, Write};

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

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsZoneDetail {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Domain")]
    domain: String,
    #[tabled(rename = "Records")]
    record_count: usize,
    #[tabled(rename = "NS Detected")]
    nameservers_detected: bool,
    #[tabled(rename = "Custom NS")]
    custom_nameservers_enabled: bool,
    #[tabled(rename = "Nameserver 1")]
    nameserver1: String,
    #[tabled(rename = "Nameserver 2")]
    nameserver2: String,
    #[tabled(rename = "SOA Email")]
    soa_email: String,
    #[tabled(rename = "Logging")]
    logging_enabled: bool,
    #[tabled(rename = "DNSSEC")]
    dns_sec_enabled: bool,
    #[tabled(rename = "Created")]
    date_created: String,
    #[tabled(rename = "Modified")]
    date_modified: String,
}

impl From<&DnsZone> for DnsZoneDetail {
    fn from(z: &DnsZone) -> Self {
        Self {
            id: z.id,
            domain: z.domain.clone(),
            record_count: z.records.len(),
            nameservers_detected: z.nameservers_detected,
            custom_nameservers_enabled: z.custom_nameservers_enabled,
            nameserver1: z.nameserver1.as_deref().unwrap_or("-").to_owned(),
            nameserver2: z.nameserver2.as_deref().unwrap_or("-").to_owned(),
            soa_email: z.soa_email.as_deref().unwrap_or("-").to_owned(),
            logging_enabled: z.logging_enabled,
            dns_sec_enabled: z.dns_sec_enabled,
            date_created: z.date_created.clone(),
            date_modified: z.date_modified.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display structs — DNS Records
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct DnsRecordRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Type")]
    record_type: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "TTL")]
    ttl: i32,
    #[tabled(rename = "Disabled")]
    disabled: bool,
}

impl From<&DnsRecord> for DnsRecordRow {
    fn from(r: &DnsRecord) -> Self {
        Self {
            id: r.id,
            record_type: r.record_type.to_string(),
            name: if r.name.is_empty() {
                "@".to_owned()
            } else {
                r.name.clone()
            },
            value: r.value.clone(),
            ttl: r.ttl,
            disabled: r.disabled,
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
) -> Result<()> {
    let api_key = auth::get_api_key()?;
    let client = if let Some(url) = auth::get_api_url() {
        CoreClient::with_base_url(api_key, url)
    } else {
        CoreClient::new(api_key)
    }
    .with_debug(debug);

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
            eprintln!("Deleted DNS zone {id}");
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
                output::print_data(&rows, format);
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
            eprintln!("Updated DNS record {record_id} in zone {zone_id}");
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
            eprintln!("Deleted DNS record {record_id} from zone {zone_id}");
        }
    }
    Ok(())
}

fn print_dns_zone(zone: &DnsZone, format: OutputFormat) {
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(zone).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = DnsZoneDetail::from(zone);
        output::print_single(&detail, format);
    }
}
