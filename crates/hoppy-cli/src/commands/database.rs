use std::io::{self, BufRead, Write};

use crate::date;
use anyhow::{Context as _, Result, bail};
use bunny_net_api::database::types::{
    Authorization, CreateDatabaseGroupPayload, CreateDatabasePayload, CreateDatabaseV2Payload,
    Database, Database2, DatabaseGroup, DatabaseV2PageInfo, ForkDatabasePayload,
    GenerateTokenDatabaseGroupPayload, GenerateTokenDatabasePayload,
    GenerateTokenDatabaseV2Payload, LimitsResponse, ListConfigResponse, ListDatabaseV2Response,
    ListVersionsDatabasePayload, PingResult, Region, RestoreVersionDatabasePayload, StorageRegion,
    UpdateDatabaseGroupPayload, UpdateDatabaseV2Payload,
};

use crate::auth;
use crate::cli::{
    DbAction, DbConfigAction, DbGroupAction, DbTokenAction, DbV2Action, OutputFormat,
    TokenAuthorization,
};
use crate::output;
use crate::redact::{RedactConfig, placeholder};

// Slug validation: lowercase letter start, then [a-z0-9-]{0,23}. Conservative
// upper bound — bunny silently 500s on long slugs (the field report saw 25
// chars fail; 13 chars passed). Adjust here if upstream changes.
const SLUG_MAX_LEN: usize = 24;

fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        bail!("slug must not be empty");
    }
    if slug.len() > SLUG_MAX_LEN {
        bail!(
            "slug too long ({} chars; max {SLUG_MAX_LEN}). Bunny silently 500s on long slugs.",
            slug.len()
        );
    }
    let bytes = slug.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_lowercase() {
        bail!("slug must start with a lowercase letter (a-z)");
    }
    for &b in &bytes[1..] {
        if !(b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-') {
            bail!("slug may only contain lowercase letters, digits, and '-'");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Display rows
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct DatabaseRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Group")]
    group_name: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Size")]
    current_size: String,
    #[tabled(rename = "URL")]
    url: String,
}

impl From<&Database> for DatabaseRow {
    fn from(d: &Database) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            group_name: d.group_name.clone(),
            version: d.version.clone(),
            current_size: d.current_size.clone(),
            url: d.url.clone(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct DatabaseDetail {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "Group ID")]
    group_id: String,
    #[tabled(rename = "Group")]
    group_name: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Block Reads")]
    block_reads: bool,
    #[tabled(rename = "Block Writes")]
    block_writes: bool,
    #[tabled(rename = "Allow Attach")]
    allow_attach: bool,
    #[tabled(rename = "Is Schema")]
    is_schema: bool,
    #[tabled(rename = "Schema")]
    schema: String,
    #[tabled(rename = "Current Size")]
    current_size: String,
    #[tabled(rename = "Size Max")]
    size_max: String,
}

impl From<&Database> for DatabaseDetail {
    fn from(d: &Database) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            url: d.url.clone(),
            group_id: d.group_id.clone(),
            group_name: d.group_name.clone(),
            version: d.version.clone(),
            block_reads: d.block_reads,
            block_writes: d.block_writes,
            allow_attach: d.allow_attach,
            is_schema: d.is_schema,
            schema: d.schema.clone().unwrap_or_else(|| "-".to_owned()),
            current_size: d.current_size.clone(),
            size_max: d.size_max.clone(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct GroupRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Storage")]
    storage_region: String,
    #[tabled(rename = "Primary")]
    primary_regions: String,
    #[tabled(rename = "Replicas")]
    replicas_regions: String,
}

impl From<&DatabaseGroup> for GroupRow {
    fn from(g: &DatabaseGroup) -> Self {
        Self {
            id: g.id.clone(),
            name: g.name.clone(),
            storage_region: g.storage_region.clone(),
            primary_regions: g.primary_regions.join(","),
            replicas_regions: g.replicas_regions.join(","),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct TokenRedactedRow {
    #[tabled(rename = "Token")]
    token: String,
    #[tabled(rename = "Length")]
    length: usize,
    #[tabled(rename = "Authorization")]
    authorization: String,
    #[tabled(rename = "Expires At")]
    expires_at: String,
}

#[derive(serde::Serialize, tabled::Tabled)]
struct TokenRevealedRow {
    #[tabled(rename = "Token")]
    token: String,
    #[tabled(rename = "Authorization")]
    authorization: String,
    #[tabled(rename = "Expires At")]
    expires_at: String,
}

#[derive(serde::Serialize, tabled::Tabled)]
struct PingRow {
    #[tabled(rename = "OK")]
    ok: bool,
    #[tabled(rename = "Latency (ms)")]
    latency_ms: u64,
    #[tabled(rename = "Error")]
    error: String,
}

impl From<&PingResult> for PingRow {
    fn from(p: &PingResult) -> Self {
        Self {
            ok: p.ok,
            latency_ms: p.latency_ms,
            error: p.error.clone().unwrap_or_else(|| "-".to_owned()),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct DatabaseV2Row {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Storage")]
    storage_region: String,
    #[tabled(rename = "Primary")]
    primary_regions: String,
    #[tabled(rename = "Replicas")]
    replicas_regions: String,
    #[tabled(rename = "Size")]
    current_size: String,
}

impl From<&Database2> for DatabaseV2Row {
    fn from(d: &Database2) -> Self {
        Self {
            id: d.id.clone(),
            name: d.name.clone(),
            storage_region: d.storage_region.clone(),
            primary_regions: d.primary_regions.join(","),
            replicas_regions: d.replicas_regions.join(","),
            current_size: format_bytes(d.current_size_bytes),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
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

#[derive(serde::Serialize, tabled::Tabled)]
struct StorageRegionRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Group")]
    group: String,
}

impl From<&StorageRegion> for StorageRegionRow {
    fn from(r: &StorageRegion) -> Self {
        Self {
            id: r.id.clone(),
            name: r.name.clone(),
            group: r.group.clone(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct RegionRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Group")]
    group: String,
    #[tabled(rename = "Lat")]
    latitude: f64,
    #[tabled(rename = "Lon")]
    longitude: f64,
}

impl From<&Region> for RegionRow {
    fn from(r: &Region) -> Self {
        Self {
            id: r.id.clone(),
            name: r.name.clone(),
            group: r.group.clone(),
            latitude: r.latitude,
            longitude: r.longitude,
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct LimitsRow {
    #[tabled(rename = "Current Databases")]
    current_databases: u32,
    #[tabled(rename = "Max Databases")]
    max_databases: u32,
}

impl From<&LimitsResponse> for LimitsRow {
    fn from(l: &LimitsResponse) -> Self {
        Self {
            current_databases: l.current_databases,
            max_databases: l.max_databases,
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct GenerationRow {
    #[tabled(rename = "Generation")]
    generation: String,
    #[tabled(rename = "Created At")]
    created_at: String,
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

fn confirm_destructive(prompt: &str, yes: bool) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    eprint!("{prompt} [y/N] ");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    let answer = line.trim().to_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(true)
    } else {
        eprintln!("Aborted.");
        Ok(false)
    }
}

#[allow(clippy::too_many_lines)]
pub async fn handle(
    action: &DbAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    quiet: bool,
    record: Option<&str>,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    let client = auth::database_client(debug, record)?;
    match action {
        DbAction::List { group_id } => {
            let resp = client.list_databases(group_id.as_deref()).await?;
            let rows: Vec<DatabaseRow> = resp.databases.iter().map(Into::into).collect();
            output::print_data(&rows, format);
        }
        DbAction::Get { id } => {
            let resp = client.get_database(id).await?;
            let detail: DatabaseDetail = (&resp.database).into();
            output::print_single(&detail, format);
        }
        DbAction::Create { slug, group } => {
            validate_slug(slug)?;
            let resp = client
                .create_database(&CreateDatabasePayload::new(slug, group))
                .await?;
            let detail: DatabaseDetail = (&resp.database).into();
            output::print_single(&detail, format);
        }
        DbAction::Delete { id } => {
            if !confirm_destructive(&format!("Delete database {id}?"), yes)? {
                return Ok(());
            }
            let resp = client.delete_database(id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "database",
                serde_json::json!({ "Database": resp.database }),
                &format!("Deleted database {}", resp.database),
            );
        }
        DbAction::Fork {
            id,
            target,
            date,
            group,
        } => {
            validate_slug(target)?;
            let resp = client
                .fork_database(
                    id,
                    &ForkDatabasePayload {
                        slug: target.clone(),
                        date: date.clone(),
                        group: group.clone(),
                    },
                )
                .await?;
            let detail: DatabaseDetail = (&resp.database).into();
            output::print_single(&detail, format);
        }
        DbAction::Restore { id, version } => {
            if !confirm_destructive(
                &format!(
                    "Restore database {id} to generation {version}? Current data is overwritten."
                ),
                yes,
            )? {
                return Ok(());
            }
            let resp = client
                .restore_database(
                    id,
                    &RestoreVersionDatabasePayload {
                        generation: version.clone(),
                    },
                )
                .await?;
            output::print_mutation_result(
                format,
                "restore",
                "database",
                serde_json::json!({ "Generation": resp.generation }),
                &format!("Restored database to generation {}", resp.generation),
            );
        }
        DbAction::Versions {
            id,
            limit,
            older_than,
            newer_than,
        } => {
            let body = ListVersionsDatabasePayload {
                limit: *limit,
                older_than: older_than.clone(),
                newer_than: newer_than.clone(),
            };
            let resp = client.list_database_versions(id, &body).await?;
            let rows: Vec<GenerationRow> = resp
                .generations
                .iter()
                .map(|g| GenerationRow {
                    generation: g.generation.clone(),
                    created_at: g.created_at.clone(),
                })
                .collect();
            output::print_data(&rows, format);
        }
        DbAction::Ping { id, token_file } => {
            let db = client.get_database(id).await?.database;
            let token = if let Some(path) = token_file {
                tokio::fs::read_to_string(path)
                    .await
                    .with_context(|| format!("reading token file {path}"))?
                    .trim()
                    .to_owned()
            } else {
                let mint = client
                    .mint_database_token(
                        id,
                        &GenerateTokenDatabasePayload::new(Authorization::ReadOnly),
                    )
                    .await
                    .context("auto-minting read-only ping token")?;
                mint.token
            };
            let result = client.ping(&db.url, &token).await;
            // `db ping` is a predicate command: exit code carries the
            // success/failure signal. Under `--quiet` we skip the payload so
            // it can be used as `if hoppy db ping --id ... --quiet; then ...`.
            if !quiet {
                output::print_single(&PingRow::from(&result), format);
            }
            if !result.ok {
                bail!("ping failed: {}", result.error.unwrap_or_default());
            }
        }
        DbAction::Statistics { id, from, to } => {
            let from = date::normalise_datetime(from)?;
            let to = date::normalise_datetime(to)?;
            let stats = client.get_database_statistics_v2(id, &from, &to).await?;
            output::print_dynamic_pascal(&stats, format);
        }
        DbAction::Usage { id, from, to } => {
            let from = date::normalise_datetime(from)?;
            let to = date::normalise_datetime(to)?;
            let usage = client.get_database_usage_v2(id, &from, &to).await?;
            output::print_dynamic_pascal(&usage, format);
        }
        DbAction::ActiveUsage => {
            let resp = client.get_active_usage_v2().await?;
            output::print_dynamic_pascal(&resp, format);
        }
        DbAction::Live { ids } => {
            if ids.is_empty() {
                bail!("at least one --id is required");
            }
            let resp = client.live_metrics_db(ids).await?;
            output::print_dynamic_pascal(&resp, format);
        }
        DbAction::V2 { action } => handle_v2(&client, action, format, yes).await?,
        DbAction::Group { action } => {
            handle_group(&client, action, format, yes, redact_cfg).await?;
        }
        DbAction::Token { action } => handle_token(&client, action, format, redact_cfg).await?,
        DbAction::Config { action } => handle_config(&client, action, format).await?,
    }
    Ok(())
}

async fn handle_v2(
    client: &bunny_net_api::database::DatabaseClient,
    action: &DbV2Action,
    format: OutputFormat,
    yes: bool,
) -> Result<()> {
    match action {
        DbV2Action::List {
            page,
            per_page,
            search,
            all,
        } => {
            if *all {
                const AUTO_PER_PAGE: u32 = 1000;
                let mut current_page: u32 = 1;
                let mut accumulated: Vec<Database2> = Vec::new();
                loop {
                    let resp = client
                        .list_databases_v2(current_page, Some(AUTO_PER_PAGE), search.as_deref())
                        .await?;
                    let has_more = resp.page_info.has_more_items;
                    if let OutputFormat::Json = format {
                        accumulated.extend(resp.databases);
                    } else {
                        let rows: Vec<DatabaseV2Row> =
                            resp.databases.iter().map(Into::into).collect();
                        output::print_data(&rows, format);
                    }
                    if !has_more {
                        break;
                    }
                    current_page += 1;
                }
                if let OutputFormat::Json = format {
                    let total_items = accumulated.len() as i64;
                    let combined = ListDatabaseV2Response {
                        databases: accumulated,
                        page_info: DatabaseV2PageInfo {
                            current_page: current_page as i64,
                            total_items,
                            has_more_items: false,
                        },
                    };
                    output::print_dynamic_pascal(&combined, format);
                }
            } else {
                let resp = client
                    .list_databases_v2(page.unwrap_or(1), *per_page, search.as_deref())
                    .await?;
                match format {
                    OutputFormat::Json => output::print_dynamic_pascal(&resp, format),
                    OutputFormat::Table | OutputFormat::Text => {
                        let rows: Vec<DatabaseV2Row> =
                            resp.databases.iter().map(Into::into).collect();
                        output::print_data(&rows, format);
                        let p = &resp.page_info;
                        eprintln!(
                            "page {} • {} total • {}",
                            p.current_page,
                            p.total_items,
                            if p.has_more_items {
                                "more pages available"
                            } else {
                                "no more pages"
                            }
                        );
                    }
                }
            }
        }
        DbV2Action::Get { id } => {
            let resp = client.get_database_v2(id).await?;
            output::print_dynamic_pascal(&resp, format);
        }
        DbV2Action::Create {
            name,
            storage_region,
            primary_regions,
            replicas_regions,
        } => {
            eprintln!(
                "warning: bunny.net Database v2 create is known to return 500 \
                 \"Internal error\" upstream as of 2026-05-05; if this fails, use \
                 `hoppy db create` (v1) instead."
            );
            let body = CreateDatabaseV2Payload::new(
                name,
                storage_region,
                primary_regions.clone(),
                replicas_regions.clone(),
            );
            let resp = client.create_database_v2(&body).await?;
            output::print_mutation_result(
                format,
                "create",
                "database-v2",
                serde_json::json!({ "DbId": resp.db_id }),
                &format!("Created database (v2) {}", resp.db_id),
            );
        }
        DbV2Action::Update {
            id,
            primary_regions,
            replicas_regions,
        } => {
            if primary_regions.is_empty() && replicas_regions.is_empty() {
                bail!(
                    "at least one update flag is required \
                     (--primary-region or --replicas-region)"
                );
            }
            let body = UpdateDatabaseV2Payload {
                primary_regions: (!primary_regions.is_empty()).then(|| primary_regions.clone()),
                replicas_regions: (!replicas_regions.is_empty()).then(|| replicas_regions.clone()),
            };
            let resp = client.update_database_v2(id, &body).await?;
            let row: DatabaseV2Row = (&resp.database).into();
            output::print_single(&row, format);
        }
        DbV2Action::Delete { id } => {
            if !confirm_destructive(&format!("Delete database {id} (v2)?"), yes)? {
                return Ok(());
            }
            let resp = client.delete_database_v2(id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "database-v2",
                serde_json::json!({ "DbId": resp.db_id }),
                &format!("Deleted database (v2) {}", resp.db_id),
            );
        }
    }
    Ok(())
}

async fn handle_group(
    client: &bunny_net_api::database::DatabaseClient,
    action: &DbGroupAction,
    format: OutputFormat,
    yes: bool,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    match action {
        DbGroupAction::List { search } => {
            let resp = client.list_groups(search.as_deref()).await?;
            let rows: Vec<GroupRow> = resp.groups.iter().map(Into::into).collect();
            output::print_data(&rows, format);
        }
        DbGroupAction::Get { id } => {
            let resp = client.get_group(id).await?;
            let row: GroupRow = (&resp.group).into();
            output::print_single(&row, format);
        }
        DbGroupAction::Create {
            display_name,
            storage_region,
            primary_regions,
            replicas_regions,
        } => {
            let body = CreateDatabaseGroupPayload::new(
                display_name,
                storage_region,
                primary_regions.clone(),
                replicas_regions.clone(),
            );
            let resp = client.create_group(&body).await?;
            let row: GroupRow = (&resp.group).into();
            output::print_single(&row, format);
        }
        DbGroupAction::Update {
            id,
            display_name,
            primary_regions,
            replicas_regions,
        } => {
            if display_name.is_none() && primary_regions.is_empty() && replicas_regions.is_empty() {
                bail!(
                    "at least one update flag is required \
                     (--display-name, --primary-region or --replicas-region)"
                );
            }
            let body = UpdateDatabaseGroupPayload {
                display_name: display_name.clone(),
                primary_regions: (!primary_regions.is_empty()).then(|| primary_regions.clone()),
                replicas_regions: (!replicas_regions.is_empty()).then(|| replicas_regions.clone()),
            };
            let resp = client.update_group(id, &body).await?;
            let row: GroupRow = (&resp.group).into();
            output::print_single(&row, format);
        }
        DbGroupAction::Delete { id } => {
            if !confirm_destructive(
                &format!("Delete database group {id}? All databases in the group are affected."),
                yes,
            )? {
                return Ok(());
            }
            let resp = client.delete_group(id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "database-group",
                serde_json::json!({ "Id": resp.group.id }),
                &format!("Deleted database group {}", resp.group.id),
            );
        }
        DbGroupAction::Stats { id, from, to } => {
            let from = date::normalise_datetime(from)?;
            let to = date::normalise_datetime(to)?;
            let stats = client.get_group_stats(id, &from, &to).await?;
            output::print_dynamic_pascal(&stats, format);
        }
        DbGroupAction::Usage { id, from, to } => {
            let from = date::normalise_datetime(from)?;
            let to = date::normalise_datetime(to)?;
            let usage = client.get_group_aggregated_usage(id, &from, &to).await?;
            output::print_dynamic_pascal(&usage, format);
        }
        DbGroupAction::Live { ids } => {
            if ids.is_empty() {
                bail!("at least one --id is required");
            }
            let resp = client.live_metrics_group(ids).await?;
            output::print_dynamic_pascal(&resp, format);
        }
        DbGroupAction::GenerateKeys {
            id,
            authorization,
            expires_at,
        } => {
            let mut body = GenerateTokenDatabaseGroupPayload::new((*authorization).into());
            if let Some(ts) = expires_at {
                body.expires_at = Some(ts.clone());
            }
            let resp = client.generate_group_keys(id, &body).await?;
            print_minted_token(
                &resp.token,
                *authorization,
                resp.expires_at.as_deref(),
                format,
                redact_cfg,
            );
        }
        DbGroupAction::InvalidateKeys { id } => {
            client.invalidate_group_keys(id).await?;
            output::print_mutation_result(
                format,
                "invalidate-keys",
                "database-group",
                serde_json::json!({ "Id": id }),
                &format!("Invalidated all keys for group {id}"),
            );
        }
    }
    Ok(())
}

async fn handle_token(
    client: &bunny_net_api::database::DatabaseClient,
    action: &DbTokenAction,
    format: OutputFormat,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    match action {
        DbTokenAction::Mint {
            db_id,
            authorization,
            expires_at,
        } => {
            let mut body = GenerateTokenDatabasePayload::new((*authorization).into());
            if let Some(ts) = expires_at {
                body.expires_at = Some(ts.clone());
            }
            let resp = client.mint_database_token(db_id, &body).await?;
            print_minted_token(
                &resp.token,
                *authorization,
                resp.expires_at.as_deref(),
                format,
                redact_cfg,
            );
        }
        DbTokenAction::Invalidate { db_id } => {
            client.invalidate_database_keys(db_id).await?;
            output::print_mutation_result(
                format,
                "invalidate-keys",
                "database-token",
                serde_json::json!({ "DbId": db_id }),
                &format!("Invalidated all keys for database {db_id}"),
            );
        }
        DbTokenAction::GenerateV2 {
            db_id,
            authorization,
            expires_at,
        } => {
            let mut body = GenerateTokenDatabaseV2Payload::new((*authorization).into());
            if let Some(ts) = expires_at {
                body.expires_at = Some(ts.clone());
            }
            let resp = client.mint_database_token_v2(db_id, &body).await?;
            print_minted_token(
                &resp.token,
                *authorization,
                resp.expires_at.as_deref(),
                format,
                redact_cfg,
            );
        }
        DbTokenAction::RevokeV2 { db_id } => {
            client.revoke_database_token_v2(db_id).await?;
            output::print_mutation_result(
                format,
                "revoke",
                "database-token-v2",
                serde_json::json!({ "DbId": db_id }),
                &format!("Revoked v2 token(s) for database {db_id}"),
            );
        }
    }
    Ok(())
}

fn print_minted_token(
    token: &str,
    authorization: TokenAuthorization,
    expires_at: Option<&str>,
    format: OutputFormat,
    redact_cfg: &RedactConfig,
) {
    let auth_str = match authorization {
        TokenAuthorization::FullAccess => "full-access",
        TokenAuthorization::ReadOnly => "read-only",
    };
    let expires = expires_at.unwrap_or("never").to_owned();
    if redact_cfg.reveal_field() {
        let row = TokenRevealedRow {
            token: token.to_owned(),
            authorization: auth_str.to_owned(),
            expires_at: expires,
        };
        output::print_single(&row, format);
    } else {
        let row = TokenRedactedRow {
            token: placeholder(token),
            length: token.chars().count(),
            authorization: auth_str.to_owned(),
            expires_at: expires,
        };
        output::print_single(&row, format);
    }
}

async fn handle_config(
    client: &bunny_net_api::database::DatabaseClient,
    action: &DbConfigAction,
    format: OutputFormat,
) -> Result<()> {
    match action {
        DbConfigAction::Show => {
            let resp = client.get_config().await?;
            render_config_show(&resp, format)?;
        }
        DbConfigAction::Limits => {
            let resp = client.get_config_limits().await?;
            match format {
                OutputFormat::Json => println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                ),
                OutputFormat::Table | OutputFormat::Text => {
                    let row: LimitsRow = (&resp).into();
                    output::print_single(&row, format);
                }
            }
        }
        DbConfigAction::Optimal { cdn_server_token } => {
            let resp = client.get_optimal(cdn_server_token).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        DbConfigAction::OptimalSingle { cdn_server_token } => {
            let resp = client.get_optimal_single(cdn_server_token).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
    }
    Ok(())
}

fn render_config_show(resp: &ListConfigResponse, format: OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(resp).context("failed to serialize to JSON")?
            );
        }
        OutputFormat::Table => {
            let storage: Vec<StorageRegionRow> = resp
                .storage_region_available
                .iter()
                .map(Into::into)
                .collect();
            let primary: Vec<RegionRow> = resp.primary_regions.iter().map(Into::into).collect();
            let replica: Vec<RegionRow> = resp.replica_regions.iter().map(Into::into).collect();
            eprintln!("Storage regions:");
            output::print_data(&storage, format);
            eprintln!("\nPrimary regions:");
            output::print_data(&primary, format);
            eprintln!("\nReplica regions:");
            output::print_data(&replica, format);
        }
        OutputFormat::Text => {
            for r in &resp.storage_region_available {
                println!("storage\t{}\t{}\t{}", r.id, r.name, r.group);
            }
            for r in &resp.primary_regions {
                println!(
                    "primary\t{}\t{}\t{}\t{}\t{}",
                    r.id, r.name, r.group, r.latitude, r.longitude
                );
            }
            for r in &resp.replica_regions {
                println!(
                    "replica\t{}\t{}\t{}\t{}\t{}",
                    r.id, r.name, r.group, r.latitude, r.longitude
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_accepts_short_lowercase() {
        assert!(validate_slug("a").is_ok());
        assert!(validate_slug("my-app").is_ok());
        assert!(validate_slug("wa-admin-prod").is_ok());
    }

    #[test]
    fn slug_rejects_long() {
        // 25 chars — failed in the field report
        assert!(validate_slug("wardrobe-assistants-admin").is_err());
    }

    #[test]
    fn slug_rejects_uppercase() {
        assert!(validate_slug("Foo").is_err());
    }

    #[test]
    fn slug_rejects_leading_digit() {
        assert!(validate_slug("1foo").is_err());
    }

    #[test]
    fn slug_rejects_underscore() {
        assert!(validate_slug("my_app").is_err());
    }

    #[test]
    fn slug_rejects_empty() {
        assert!(validate_slug("").is_err());
    }
}
