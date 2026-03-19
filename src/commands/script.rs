use crate::auth;
use crate::cli::{
    OutputFormat, ScriptAction, ScriptCodeAction, ScriptReleaseAction, ScriptSecretAction,
    ScriptVariableAction,
};
use crate::output::{self, PaginatedListJson};
use anyhow::{Result, bail};
use bunny_api_compute::ComputeClient;
use bunny_api_compute::{
    AddSecret, AddVariable, CreateEdgeScript, EdgeScript, EdgeScriptCode, EdgeScriptRelease,
    EdgeScriptSecret, EdgeScriptStatistics, EdgeScriptVariable, PublishScript, ScriptType,
    SecretList, UpdateEdgeScript, UpdateSecret, UpdateVariable, UpsertSecret, UpsertVariable,
};
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Helper: build the client
// ---------------------------------------------------------------------------

fn client(debug: bool) -> Result<ComputeClient> {
    Ok(if let Some(url) = auth::get_api_url() {
        ComputeClient::with_base_url(auth::get_api_key()?, url).with_debug(debug)
    } else {
        ComputeClient::new(auth::get_api_key()?).with_debug(debug)
    })
}

// ---------------------------------------------------------------------------
// ScriptType display helper
// ---------------------------------------------------------------------------

fn script_type_str(t: ScriptType) -> &'static str {
    match t {
        ScriptType::Dns => "Dns",
        ScriptType::Cdn => "Cdn",
        ScriptType::Middleware => "Middleware",
    }
}

fn u8_to_script_type(n: u8) -> Result<ScriptType> {
    serde_json::from_value(serde_json::json!(n)).map_err(|_| {
        anyhow::anyhow!("invalid script-type {n}: must be 0 (Dns), 1 (Cdn), or 2 (Middleware)")
    })
}

// ---------------------------------------------------------------------------
// Display rows — Scripts
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ScriptRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    script_type: String,
    #[tabled(rename = "Last Modified")]
    last_modified: String,
    #[tabled(rename = "Hostname")]
    default_hostname: String,
}

impl From<&EdgeScript> for ScriptRow {
    fn from(s: &EdgeScript) -> Self {
        Self {
            id: s.id,
            name: s.name.as_deref().unwrap_or("-").to_owned(),
            script_type: script_type_str(s.script_type).to_owned(),
            last_modified: s.last_modified.clone(),
            default_hostname: s.default_hostname.as_deref().unwrap_or("-").to_owned(),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct ScriptDetail {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    script_type: String,
    #[tabled(rename = "Last Modified")]
    last_modified: String,
    #[tabled(rename = "Hostname")]
    default_hostname: String,
    #[tabled(rename = "System Hostname")]
    system_hostname: String,
    #[tabled(rename = "Monthly Cost")]
    monthly_cost: String,
    #[tabled(rename = "Monthly Requests")]
    monthly_request_count: i64,
    #[tabled(rename = "Current Release ID")]
    current_release_id: i64,
}

impl From<&EdgeScript> for ScriptDetail {
    fn from(s: &EdgeScript) -> Self {
        Self {
            id: s.id,
            name: s.name.as_deref().unwrap_or("-").to_owned(),
            script_type: script_type_str(s.script_type).to_owned(),
            last_modified: s.last_modified.clone(),
            default_hostname: s.default_hostname.as_deref().unwrap_or("-").to_owned(),
            system_hostname: s.system_hostname.as_deref().unwrap_or("-").to_owned(),
            monthly_cost: format!("{:.4}", s.monthly_cost),
            monthly_request_count: s.monthly_request_count,
            current_release_id: s.current_release_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Code
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ScriptCodeRow {
    #[tabled(rename = "Last Modified")]
    last_modified: String,
    #[tabled(rename = "Code (truncated)")]
    code_preview: String,
}

impl From<&EdgeScriptCode> for ScriptCodeRow {
    fn from(c: &EdgeScriptCode) -> Self {
        let preview = c
            .code
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(80)
            .collect::<String>();
        Self {
            last_modified: c.last_modified.clone(),
            code_preview: preview,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Releases
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ReleaseRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "UUID")]
    uuid: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Note")]
    note: String,
    #[tabled(rename = "Published")]
    date_published: String,
}

impl From<&EdgeScriptRelease> for ReleaseRow {
    fn from(r: &EdgeScriptRelease) -> Self {
        let status = match r.status {
            bunny_api_compute::ReleaseStatus::Live => "Live",
            bunny_api_compute::ReleaseStatus::Archived => "Archived",
        };
        Self {
            id: r.id,
            uuid: r.uuid.as_deref().unwrap_or("-").to_owned(),
            status: status.to_owned(),
            note: r.note.as_deref().unwrap_or("-").to_owned(),
            date_published: r.date_published.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Variables
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct VariableRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Required")]
    required: bool,
    #[tabled(rename = "Default Value")]
    default_value: String,
}

impl From<&EdgeScriptVariable> for VariableRow {
    fn from(v: &EdgeScriptVariable) -> Self {
        Self {
            id: v.id,
            name: v.name.as_deref().unwrap_or("-").to_owned(),
            required: v.required,
            default_value: v.default_value.as_deref().unwrap_or("-").to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Secrets
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct SecretRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Last Modified")]
    last_modified: String,
}

impl From<&EdgeScriptSecret> for SecretRow {
    fn from(s: &EdgeScriptSecret) -> Self {
        Self {
            id: s.id,
            name: s.name.as_deref().unwrap_or("-").to_owned(),
            last_modified: s.last_modified.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Statistics
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct StatisticsRow {
    #[tabled(rename = "Total Requests")]
    total_requests_served: i64,
    #[tabled(rename = "Total CPU Used")]
    total_cpu_used: String,
    #[tabled(rename = "Monthly Cost")]
    total_monthly_cost: String,
    #[tabled(rename = "Avg CPU/Execution")]
    average_cpu_time_per_execution: String,
}

impl From<&EdgeScriptStatistics> for StatisticsRow {
    fn from(s: &EdgeScriptStatistics) -> Self {
        Self {
            total_requests_served: s.total_requests_served,
            total_cpu_used: format!("{:.4}", s.total_cpu_used),
            total_monthly_cost: format!("{:.4}", s.total_monthly_cost),
            average_cpu_time_per_execution: format!("{:.4}", s.average_cpu_time_per_execution),
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &ScriptAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    match action {
        ScriptAction::List {
            search,
            page,
            per_page,
        } => handle_list(search.as_deref(), *page, *per_page, format, debug).await,
        ScriptAction::Get { id } => handle_get(*id, format, debug).await,
        ScriptAction::Create {
            name,
            script_type,
            code,
            create_linked_pull_zone,
            linked_pull_zone_name,
        } => {
            handle_create(
                name,
                *script_type,
                code.as_deref(),
                *create_linked_pull_zone,
                linked_pull_zone_name.as_deref(),
                format,
                debug,
            )
            .await
        }
        ScriptAction::Update {
            id,
            name,
            script_type,
        } => handle_update(*id, name.as_deref(), *script_type, format, debug).await,
        ScriptAction::Delete {
            id,
            delete_linked_pull_zones,
        } => handle_delete(*id, *delete_linked_pull_zones, yes, debug).await,
        ScriptAction::Code { action } => handle_code(action, format, debug).await,
        ScriptAction::Publish { id, note } => handle_publish(*id, note.as_deref(), debug).await,
        ScriptAction::Release { action } => handle_release(action, format, debug).await,
        ScriptAction::Variable { action } => handle_variable(action, format, debug, yes).await,
        ScriptAction::Secret { action } => handle_secret(action, format, debug, yes).await,
        ScriptAction::Statistics {
            id,
            date_from,
            date_to,
            hourly,
        } => {
            handle_statistics(
                *id,
                date_from.as_deref(),
                date_to.as_deref(),
                *hourly,
                format,
                debug,
            )
            .await
        }
        ScriptAction::RotateDeploymentKey { id } => {
            if !yes {
                eprint!(
                    "Rotate deployment key for script {id}? The old key will stop working immediately. [y/N] "
                );
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let c = client(debug)?;
            c.rotate_deployment_key(*id).await?;
            eprintln!("Rotated deployment key for script {id}");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Script sub-handlers
// ---------------------------------------------------------------------------

async fn handle_list(
    search: Option<&str>,
    page: Option<i32>,
    per_page: Option<i32>,
    format: OutputFormat,
    debug: bool,
) -> Result<()> {
    let c = client(debug)?;
    let result = c.list_scripts(page, per_page, search).await?;
    if let OutputFormat::Json = format {
        let envelope = PaginatedListJson {
            items: &result.items,
            current_page: result.current_page as i64,
            total_items: result.total_items as i64,
            has_more_items: result.has_more_items,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON")
        );
    } else {
        let rows: Vec<ScriptRow> = result.items.iter().map(ScriptRow::from).collect();
        output::print_data(&rows, format);
    }
    Ok(())
}

async fn handle_get(id: i64, format: OutputFormat, debug: bool) -> Result<()> {
    let c = client(debug)?;
    let script = c.get_script(id).await?;
    if let OutputFormat::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(&script).expect("failed to serialize to JSON")
        );
    } else {
        let row = ScriptDetail::from(&script);
        output::print_single(&row, format);
    }
    Ok(())
}

async fn handle_create(
    name: &str,
    script_type: u8,
    code: Option<&str>,
    create_linked_pull_zone: bool,
    linked_pull_zone_name: Option<&str>,
    format: OutputFormat,
    debug: bool,
) -> Result<()> {
    let c = client(debug)?;
    let body = CreateEdgeScript {
        name: Some(name.to_owned()),
        code: code.map(str::to_owned),
        script_type: u8_to_script_type(script_type)?,
        create_linked_pull_zone,
        linked_pull_zone_name: linked_pull_zone_name.map(str::to_owned),
        integration: None,
    };
    let script = c.create_script(&body).await?;
    if let OutputFormat::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(&script).expect("failed to serialize to JSON")
        );
    } else {
        let row = ScriptDetail::from(&script);
        output::print_single(&row, format);
    }
    Ok(())
}

async fn handle_update(
    id: i64,
    name: Option<&str>,
    script_type: Option<u8>,
    format: OutputFormat,
    debug: bool,
) -> Result<()> {
    if name.is_none() && script_type.is_none() {
        bail!("at least one update flag is required (--name, --script-type)");
    }
    let c = client(debug)?;
    let body = UpdateEdgeScript {
        name: name.map(str::to_owned),
        script_type: script_type.map(u8_to_script_type).transpose()?,
    };
    let script = c.update_script(id, &body).await?;
    if let OutputFormat::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(&script).expect("failed to serialize to JSON")
        );
    } else {
        let row = ScriptDetail::from(&script);
        output::print_single(&row, format);
    }
    Ok(())
}

async fn handle_delete(
    id: i64,
    delete_linked_pull_zones: bool,
    yes: bool,
    debug: bool,
) -> Result<()> {
    if !yes {
        eprint!("Delete script {id}? [y/N] ");
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        if !is_confirmed(&line) {
            eprintln!("Aborted.");
            return Ok(());
        }
    }
    let c = client(debug)?;
    c.delete_script(id, delete_linked_pull_zones).await?;
    eprintln!("Deleted script {id}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Code sub-handlers
// ---------------------------------------------------------------------------

async fn handle_code(action: &ScriptCodeAction, format: OutputFormat, debug: bool) -> Result<()> {
    let c = client(debug)?;
    match action {
        ScriptCodeAction::Get { id } => {
            let code = c.get_script_code(*id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&code).expect("failed to serialize to JSON")
                );
            } else {
                let row = ScriptCodeRow::from(&code);
                output::print_single(&row, format);
            }
        }
        ScriptCodeAction::Update { id, code, file } => {
            let source = match (code.as_deref(), file.as_deref()) {
                (Some(c), _) => c.to_owned(),
                (_, Some(path)) => tokio::fs::read_to_string(path)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to read file {path}: {e}"))?,
                (None, None) => bail!("one of --code or --file is required"),
            };
            c.update_script_code(*id, &source).await?;
            eprintln!("Updated code for script {id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Publish handler
// ---------------------------------------------------------------------------

async fn handle_publish(id: i64, note: Option<&str>, debug: bool) -> Result<()> {
    let c = client(debug)?;
    let body = PublishScript {
        note: note.map(str::to_owned),
    };
    c.publish_script(id, &body).await?;
    eprintln!("Published script {id}");
    Ok(())
}

// ---------------------------------------------------------------------------
// Release sub-handlers
// ---------------------------------------------------------------------------

async fn handle_release(
    action: &ScriptReleaseAction,
    format: OutputFormat,
    debug: bool,
) -> Result<()> {
    let c = client(debug)?;
    match action {
        ScriptReleaseAction::List { id, page, per_page } => {
            let result = c.list_releases(*id, *page, *per_page).await?;
            if let OutputFormat::Json = format {
                let envelope = PaginatedListJson {
                    items: &result.items,
                    current_page: result.current_page as i64,
                    total_items: result.total_items as i64,
                    has_more_items: result.has_more_items,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON")
                );
            } else {
                let rows: Vec<ReleaseRow> = result.items.iter().map(ReleaseRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ScriptReleaseAction::GetActive { id } => {
            let release = c.get_active_release(*id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&release).expect("failed to serialize to JSON")
                );
            } else {
                let row = ReleaseRow::from(&release);
                output::print_single(&row, format);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Variable sub-handlers
// ---------------------------------------------------------------------------

async fn handle_variable(
    action: &ScriptVariableAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    let c = client(debug)?;
    match action {
        ScriptVariableAction::List { id } => {
            let script = c.get_script(*id).await?;
            let vars = script.edge_script_variables.unwrap_or_default();
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&vars).expect("failed to serialize to JSON")
                );
            } else {
                let rows: Vec<VariableRow> = vars.iter().map(VariableRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ScriptVariableAction::Add {
            id,
            name,
            required,
            default_value,
        } => {
            let body = AddVariable {
                name: name.clone(),
                required: *required,
                default_value: default_value.clone(),
            };
            let var = c.add_variable(*id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&var).expect("failed to serialize to JSON")
                );
            } else {
                let row = VariableRow::from(&var);
                output::print_single(&row, format);
            }
        }
        ScriptVariableAction::Update {
            id,
            variable_id,
            required,
            default_value,
        } => {
            if required.is_none() && default_value.is_none() {
                bail!("at least one update flag is required (--required, --default-value)");
            }
            let body = UpdateVariable {
                required: *required,
                default_value: default_value.clone(),
            };
            let var = c.update_variable(*id, *variable_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&var).expect("failed to serialize to JSON")
                );
            } else {
                let row = VariableRow::from(&var);
                output::print_single(&row, format);
            }
        }
        ScriptVariableAction::Delete { id, variable_id } => {
            if !yes {
                eprint!("Delete variable {variable_id} from script {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            c.delete_variable(*id, *variable_id).await?;
            eprintln!("Deleted variable {variable_id} from script {id}");
        }
        ScriptVariableAction::Upsert {
            id,
            name,
            required,
            default_value,
        } => {
            let body = UpsertVariable {
                name: name.clone(),
                required: *required,
                default_value: default_value.clone(),
            };
            let var = c.upsert_variable(*id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&var).expect("failed to serialize to JSON")
                );
            } else {
                let row = VariableRow::from(&var);
                output::print_single(&row, format);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Secret sub-handlers
// ---------------------------------------------------------------------------

async fn handle_secret(
    action: &ScriptSecretAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    let c = client(debug)?;
    match action {
        ScriptSecretAction::List { id } => {
            let result: SecretList = c.list_secrets(*id).await?;
            let secrets = result.secrets.unwrap_or_default();
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&secrets).expect("failed to serialize to JSON")
                );
            } else {
                let rows: Vec<SecretRow> = secrets.iter().map(SecretRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ScriptSecretAction::Add { id, name, value } => {
            let body = AddSecret {
                name: name.clone(),
                secret: Some(value.clone()),
            };
            let secret = c.add_secret(*id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&secret).expect("failed to serialize to JSON")
                );
            } else {
                let row = SecretRow::from(&secret);
                output::print_single(&row, format);
            }
        }
        ScriptSecretAction::Update {
            id,
            secret_id,
            value,
        } => {
            let body = UpdateSecret {
                secret: Some(value.clone()),
            };
            let secret = c.update_secret(*id, *secret_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&secret).expect("failed to serialize to JSON")
                );
            } else {
                let row = SecretRow::from(&secret);
                output::print_single(&row, format);
            }
        }
        ScriptSecretAction::Delete { id, secret_id } => {
            if !yes {
                eprint!("Delete secret {secret_id} from script {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            c.delete_secret(*id, *secret_id).await?;
            eprintln!("Deleted secret {secret_id} from script {id}");
        }
        ScriptSecretAction::Upsert { id, name, value } => {
            let body = UpsertSecret {
                name: Some(name.clone()),
                secret: Some(value.clone()),
            };
            let secret = c.upsert_secret(*id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&secret).expect("failed to serialize to JSON")
                );
            } else {
                let row = SecretRow::from(&secret);
                output::print_single(&row, format);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Statistics handler
// ---------------------------------------------------------------------------

async fn handle_statistics(
    id: i64,
    date_from: Option<&str>,
    date_to: Option<&str>,
    hourly: bool,
    format: OutputFormat,
    debug: bool,
) -> Result<()> {
    let c = client(debug)?;
    let stats = c
        .get_script_statistics(id, date_from, date_to, hourly)
        .await?;
    if let OutputFormat::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(&stats).expect("failed to serialize to JSON")
        );
    } else {
        let row = StatisticsRow::from(&stats);
        output::print_single(&row, format);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_confirmed(input: &str) -> bool {
    let answer = input.trim().to_lowercase();
    answer == "y" || answer == "yes"
}
