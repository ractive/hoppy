use std::io::{self, BufRead, Write};

use crate::date;
use anyhow::{Context as _, Result, bail};
use bunny_net_api::database::types::{
    Authorization, CreateDatabaseGroupPayload, CreateDatabasePayload, CreateDatabaseV2Payload,
    Database, DatabaseGroup, ForkDatabasePayload, GenerateTokenDatabaseGroupPayload,
    GenerateTokenDatabasePayload, GenerateTokenDatabaseV2Payload, ListVersionsDatabasePayload,
    PingResult, RestoreVersionDatabasePayload,
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
        DbAction::Fork { id, target, group } => {
            validate_slug(target)?;
            // Resolve a default group if the user didn't pass one.
            let group_id = if let Some(g) = group {
                g.clone()
            } else {
                let src = client.get_database(id).await?;
                src.database.group_id
            };
            let resp = client
                .fork_database(
                    id,
                    &ForkDatabasePayload {
                        slug: target.clone(),
                        group: group_id,
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
        DbAction::Versions { id, limit } => {
            let body = ListVersionsDatabasePayload {
                limit: *limit,
                older_than: None,
                newer_than: None,
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
            output::print_single(&PingRow::from(&result), format);
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
        } => {
            let resp = client
                .list_databases_v2(*page, *per_page, search.as_deref())
                .await?;
            output::print_dynamic_pascal(&resp, format);
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
    let _ = format; // Config responses are nested → JSON only.
    match action {
        DbConfigAction::Show => {
            let resp = client.get_config().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        DbConfigAction::Limits => {
            let resp = client.get_config_limits().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        DbConfigAction::Optimal => {
            let resp = client.get_optimal().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        DbConfigAction::OptimalSingle => {
            anyhow::bail!(
                "`db config optimal-single` is broken upstream (bunny.net returns HTTP 400 \
                 — missing field `cdn_server_token`). The subcommand is hidden until upstream \
                 fixes the route."
            );
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
