use crate::auth;
use crate::cli::{
    OutputFormat, ShieldAccessListAction, ShieldAction, ShieldApiGuardianAction,
    ShieldBotDetectionAction, ShieldMetricsAction, ShieldRateLimitAction,
    ShieldUploadScanningAction, ShieldWafAction, ShieldZoneAction,
};
use crate::output::{self, PaginatedListJson};
use anyhow::{Context, Result, bail};
use bunny_api_shield::types::{
    AccessListAction, AccessListDetails, AccessListType, BotDetectionConfigurationState,
    BotDetectionExecutionMode, BotDetectionSensitivity, BrowserFingerprintAggression,
    BrowserFingerprintConfiguration, CreateCustomAccessList, CreateCustomWafRule,
    CreateRateLimitRule, CustomAccessList, CustomWafRule, DdosExecutionMode, DdosShieldSensitivity,
    EventLog, IpAddressConfiguration, RateLimitCounterKey, RateLimitRule,
    RateLimitRuleConfiguration, RequestIntegrityConfiguration, ReviewActionType,
    ShieldZonePullZoneMapping, ShieldZoneRequest, ShieldZoneResponse, TriggeredRuleItem,
    UpdateAccessListConfiguration, UpdateApiGuardianEndpointRequest, UpdateApiGuardianRequest,
    UpdateBotDetection, UpdateCustomAccessList, UpdateCustomWafRule, UpdateRateLimitRule,
    UpdateReviewTriggeredRuleRequest, UpdateShieldZoneRequest,
    UpdateUploadScanningConfigurationRequest, UploadOpenApiSpecificationRequest,
    UploadScanningConfigurationState, UploadScanningScannerMode, WafExecutionMode,
    WafProfileMinimal, WafRuleActionType, WafRuleConfiguration, WafRuleOperatorType,
    WafRuleSeverityType,
};
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Display rows — Shield Zones
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ShieldZoneRow {
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: i64,
    #[tabled(rename = "Pull Zone ID")]
    pull_zone_id: String,
    #[tabled(rename = "WAF Enabled")]
    waf_enabled: String,
    #[tabled(rename = "Plan")]
    plan_type: String,
    #[tabled(rename = "DDoS Sensitivity")]
    ddos_sensitivity: String,
}

impl From<&ShieldZoneResponse> for ShieldZoneRow {
    fn from(z: &ShieldZoneResponse) -> Self {
        Self {
            shield_zone_id: z.shield_zone_id,
            pull_zone_id: z
                .pull_zone_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            waf_enabled: z
                .waf_enabled
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            plan_type: z
                .plan_type
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            ddos_sensitivity: z
                .d_do_s_shield_sensitivity
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — WAF rules
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct WafRuleRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: i64,
    #[tabled(rename = "Name")]
    rule_name: String,
    #[tabled(rename = "Action")]
    action_type: String,
}

impl From<&CustomWafRule> for WafRuleRow {
    fn from(r: &CustomWafRule) -> Self {
        Self {
            id: r.id,
            shield_zone_id: r.shield_zone_id,
            rule_name: r.rule_name.as_deref().unwrap_or("-").to_owned(),
            action_type: r
                .rule_configuration
                .as_ref()
                .map(|c| c.action_type.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — WAF profiles
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct WafProfileRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Category")]
    profile_category: String,
    #[tabled(rename = "Premium")]
    is_premium: bool,
    #[tabled(rename = "Description")]
    description: String,
}

impl From<&WafProfileMinimal> for WafProfileRow {
    fn from(p: &WafProfileMinimal) -> Self {
        Self {
            id: p.id,
            name: p.name.as_deref().unwrap_or("-").to_owned(),
            profile_category: p.profile_category.as_deref().unwrap_or("-").to_owned(),
            is_premium: p.is_premium,
            description: p.description.as_deref().unwrap_or("-").to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Rate limit rules
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct RateLimitRuleRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: i64,
    #[tabled(rename = "Name")]
    rule_name: String,
    #[tabled(rename = "Requests")]
    request_count: String,
    #[tabled(rename = "Timeframe")]
    timeframe: String,
}

impl From<&RateLimitRule> for RateLimitRuleRow {
    fn from(r: &RateLimitRule) -> Self {
        Self {
            id: r.id,
            shield_zone_id: r.shield_zone_id,
            rule_name: r.rule_name.as_deref().unwrap_or("-").to_owned(),
            request_count: r
                .rule_configuration
                .as_ref()
                .map(|c| c.request_count.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            timeframe: r
                .rule_configuration
                .as_ref()
                .map(|c| c.timeframe.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Access lists
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct AccessListRow {
    #[tabled(rename = "List ID")]
    list_id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    list_type: String,
    #[tabled(rename = "Enabled")]
    is_enabled: bool,
    #[tabled(rename = "Action")]
    action: String,
    #[tabled(rename = "Entries")]
    entry_count: i64,
}

impl From<&AccessListDetails> for AccessListRow {
    fn from(l: &AccessListDetails) -> Self {
        Self {
            list_id: l.list_id,
            name: l.name.as_deref().unwrap_or("-").to_owned(),
            list_type: l.list_type.to_string(),
            is_enabled: l.is_enabled,
            action: l.action.to_string(),
            entry_count: l.entry_count,
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct CustomAccessListRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    list_type: String,
    #[tabled(rename = "Entries")]
    entry_count: String,
}

impl From<&CustomAccessList> for CustomAccessListRow {
    fn from(l: &CustomAccessList) -> Self {
        Self {
            id: l.id,
            name: l.name.as_deref().unwrap_or("-").to_owned(),
            list_type: l.list_type.to_string(),
            entry_count: l
                .entry_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Bot detection
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct BotDetectionRow {
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: i64,
    #[tabled(rename = "Execution Mode")]
    execution_mode: String,
    #[tabled(rename = "Request Integrity")]
    request_integrity: String,
    #[tabled(rename = "IP Address")]
    ip_address: String,
    #[tabled(rename = "Fingerprint")]
    fingerprint: String,
}

impl From<&BotDetectionConfigurationState> for BotDetectionRow {
    fn from(s: &BotDetectionConfigurationState) -> Self {
        Self {
            shield_zone_id: s.shield_zone_id,
            execution_mode: s
                .execution_mode
                .map(|m| m.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            request_integrity: s
                .request_integrity
                .sensitivity
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            ip_address: s
                .ip_address
                .sensitivity
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            fingerprint: s
                .browser_fingerprint
                .sensitivity
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — API Guardian endpoints
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ApiGuardianEndpointRow {
    #[tabled(rename = "Endpoint ID")]
    endpoint_id: String,
    #[tabled(rename = "Path")]
    request_path: String,
    #[tabled(rename = "Methods")]
    request_methods: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
    #[tabled(rename = "Validate Req")]
    validate_request: String,
    #[tabled(rename = "Validate Resp")]
    validate_response: String,
}

impl From<&bunny_api_shield::types::ApiGuardianEndpoint> for ApiGuardianEndpointRow {
    fn from(e: &bunny_api_shield::types::ApiGuardianEndpoint) -> Self {
        Self {
            endpoint_id: e
                .api_guardian_endpoint_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            request_path: e.request_path.as_deref().unwrap_or("-").to_owned(),
            request_methods: e.request_methods.as_deref().unwrap_or("-").to_owned(),
            enabled: e
                .enabled
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            validate_request: e
                .validate_request_body_schema
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            validate_response: e
                .validate_response_body_schema
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Upload Scanning
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct UploadScanningRow {
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: String,
    #[tabled(rename = "Enabled")]
    is_enabled: String,
    #[tabled(rename = "Antivirus Mode")]
    antivirus_mode: String,
    #[tabled(rename = "CSAM Mode")]
    csam_mode: String,
}

impl From<&UploadScanningConfigurationState> for UploadScanningRow {
    fn from(s: &UploadScanningConfigurationState) -> Self {
        Self {
            shield_zone_id: s
                .shield_zone_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            is_enabled: s
                .is_enabled
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            antivirus_mode: s
                .antivirus_scanning_mode
                .map(|m| m.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            csam_mode: s
                .csam_scanning_mode
                .map(|m| m.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Event Logs
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct EventLogRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Rule ID")]
    rule_id: String,
    #[tabled(rename = "Method")]
    method: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Country")]
    country: String,
    #[tabled(rename = "Log")]
    log_snippet: String,
}

impl From<&EventLog> for EventLogRow {
    fn from(e: &EventLog) -> Self {
        let labels = e.labels.as_ref();
        let log_snippet = e
            .log
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect::<String>();
        Self {
            timestamp: e
                .timestamp
                .map(|t| t.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            rule_id: labels
                .and_then(|l| l.rule_id.as_deref())
                .unwrap_or("-")
                .to_owned(),
            method: labels
                .and_then(|l| l.method.as_deref())
                .unwrap_or("-")
                .to_owned(),
            status: labels
                .and_then(|l| l.status.as_deref())
                .unwrap_or("-")
                .to_owned(),
            country: labels
                .and_then(|l| l.country.as_deref())
                .unwrap_or("-")
                .to_owned(),
            log_snippet,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Triggered WAF rules
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct TriggeredRuleRow {
    #[tabled(rename = "Rule ID")]
    rule_id: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Total Triggers")]
    total_triggered: String,
}

impl From<&TriggeredRuleItem> for TriggeredRuleRow {
    fn from(r: &TriggeredRuleItem) -> Self {
        Self {
            rule_id: r.rule_id.as_deref().unwrap_or("-").to_owned(),
            description: r.rule_description.as_deref().unwrap_or("-").to_owned(),
            total_triggered: r
                .total_triggered_requests
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Pullzone mapping
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct PullzoneMappingRow {
    #[tabled(rename = "Shield Zone ID")]
    shield_zone_id: String,
    #[tabled(rename = "Pull Zone ID")]
    pull_zone_id: String,
}

impl From<&ShieldZonePullZoneMapping> for PullzoneMappingRow {
    fn from(m: &ShieldZonePullZoneMapping) -> Self {
        Self {
            shield_zone_id: m
                .shield_zone_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            pull_zone_id: m
                .pull_zone_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Enum conversion helpers (integer -> serde_repr enum via serde_json)
// ---------------------------------------------------------------------------

fn u8_to_enum<T: serde::de::DeserializeOwned>(n: u8, type_name: &str) -> Result<T> {
    serde_json::from_value(serde_json::json!(n))
        .map_err(|_| anyhow::anyhow!("invalid value {n} for {type_name}"))
}

fn u16_to_enum<T: serde::de::DeserializeOwned>(n: u16, type_name: &str) -> Result<T> {
    serde_json::from_value(serde_json::json!(n))
        .map_err(|_| anyhow::anyhow!("invalid value {n} for {type_name}"))
}

// ---------------------------------------------------------------------------
// Top-level handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &ShieldAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        ShieldAction::Zone { action } => handle_zone(action, format, debug, record).await,
        ShieldAction::Waf { action } => handle_waf(action, format, debug, yes, record).await,
        ShieldAction::RateLimit { action } => {
            handle_rate_limit(action, format, debug, yes, record).await
        }
        ShieldAction::AccessList { action } => {
            handle_access_list(action, format, debug, yes, record).await
        }
        ShieldAction::BotDetection { action } => {
            handle_bot_detection(action, format, debug, record).await
        }
        ShieldAction::Metrics { action } => handle_metrics(action, format, debug, record).await,
        ShieldAction::ApiGuardian { action } => {
            handle_api_guardian(action, format, debug, record).await
        }
        ShieldAction::UploadScanning { action } => {
            handle_upload_scanning(action, format, debug, record).await
        }
        ShieldAction::EventLogs {
            shield_zone_id,
            date,
            continuation_token,
            all,
        } => {
            handle_event_logs(
                *shield_zone_id,
                date,
                continuation_token.as_deref(),
                *all,
                format,
                debug,
                record,
            )
            .await
        }
        ShieldAction::PullzoneMapping => handle_pullzone_mapping(format, debug, record).await,
    }
}

// ---------------------------------------------------------------------------
// Shield Zone handler
// ---------------------------------------------------------------------------

async fn handle_zone(
    action: &ShieldZoneAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldZoneAction::List => {
            let result = client.list_shield_zones().await?;
            let zones = result.data.unwrap_or_default();
            if let OutputFormat::Json = format {
                let page = result.page;
                let envelope = PaginatedListJson {
                    items: &zones,
                    current_page: page.as_ref().map(|p| p.current_page as i64).unwrap_or(1),
                    total_items: page
                        .as_ref()
                        .map(|p| p.total_count as i64)
                        .unwrap_or(zones.len() as i64),
                    has_more_items: page
                        .as_ref()
                        .map(|p| p.next_page.is_some() || p.current_page < p.total_pages)
                        .unwrap_or(false),
                };
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<ShieldZoneRow> = zones.iter().map(ShieldZoneRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldZoneAction::Get { shield_zone_id } => {
            let zone = client.get_shield_zone(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&zone).expect("failed to serialize to JSON")
                );
            } else {
                let row = ShieldZoneRow::from(&zone);
                output::print_single(&row, format);
            }
        }
        ShieldZoneAction::GetByPullzone { pull_zone_id } => {
            let zone = client.get_shield_zone_by_pull_zone(*pull_zone_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&zone).expect("failed to serialize to JSON")
                );
            } else {
                let row = ShieldZoneRow::from(&zone);
                output::print_single(&row, format);
            }
        }
        ShieldZoneAction::Create { pull_zone_id } => {
            let zone = client.create_shield_zone(*pull_zone_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&zone).expect("failed to serialize to JSON")
                );
            } else {
                let row = ShieldZoneRow::from(&zone);
                output::print_single(&row, format);
            }
        }
        ShieldZoneAction::Update {
            shield_zone_id,
            waf_enabled,
            waf_execution_mode,
            ddos_sensitivity,
            ddos_execution_mode,
            ddos_challenge_window,
            learning_mode,
        } => {
            if waf_enabled.is_none()
                && waf_execution_mode.is_none()
                && ddos_sensitivity.is_none()
                && ddos_execution_mode.is_none()
                && ddos_challenge_window.is_none()
                && learning_mode.is_none()
            {
                bail!(
                    "at least one update flag is required (--waf-enabled, --waf-execution-mode, --ddos-sensitivity, --ddos-execution-mode, --ddos-challenge-window, --learning-mode)"
                );
            }

            let mut zone_req = ShieldZoneRequest::default();

            if let Some(v) = waf_enabled {
                zone_req.waf_enabled = Some(*v);
            }
            if let Some(v) = waf_execution_mode {
                zone_req.waf_execution_mode =
                    Some(u8_to_enum::<WafExecutionMode>(*v, "waf-execution-mode")?);
            }
            if let Some(v) = ddos_sensitivity {
                zone_req.d_do_s_shield_sensitivity =
                    Some(u8_to_enum::<DdosShieldSensitivity>(*v, "ddos-sensitivity")?);
            }
            if let Some(v) = ddos_execution_mode {
                zone_req.d_do_s_execution_mode =
                    Some(u8_to_enum::<DdosExecutionMode>(*v, "ddos-execution-mode")?);
            }
            if let Some(v) = ddos_challenge_window {
                zone_req.d_do_s_challenge_window = Some(*v);
            }
            if let Some(v) = learning_mode {
                zone_req.learning_mode = Some(*v);
            }

            let body = UpdateShieldZoneRequest {
                shield_zone_id: *shield_zone_id,
                shield_zone: Some(zone_req),
            };
            client.update_shield_zone(body).await?;
            eprintln!("Updated Shield Zone {shield_zone_id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// WAF handler
// ---------------------------------------------------------------------------

async fn handle_waf(
    action: &ShieldWafAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldWafAction::Profiles => {
            let profiles = client.list_waf_profiles().await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&profiles).expect("failed to serialize to JSON")
                );
            } else {
                let rows: Vec<WafProfileRow> = profiles.iter().map(WafProfileRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldWafAction::ListRules { shield_zone_id } => {
            let rules = client.list_waf_rules(*shield_zone_id).await?;
            let rows: Vec<WafRuleRow> = rules.iter().map(WafRuleRow::from).collect();
            output::print_data(&rows, format);
        }
        ShieldWafAction::GetRule { id } => {
            let rule = client.get_waf_rule(*id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = WafRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldWafAction::AddRule {
            shield_zone_id,
            name,
            action_type,
            operator_type,
            severity_type,
            value,
        } => {
            let config = WafRuleConfiguration {
                action_type: u8_to_enum::<WafRuleActionType>(*action_type, "action-type")?,
                variable_types: Some(Default::default()),
                operator_type: u8_to_enum::<WafRuleOperatorType>(*operator_type, "operator-type")?,
                severity_type: u8_to_enum::<WafRuleSeverityType>(*severity_type, "severity-type")?,
                transformation_types: Some(vec![]),
                value: value.clone(),
                chained_rule_conditions: None,
            };
            let body = CreateCustomWafRule {
                shield_zone_id: *shield_zone_id,
                rule_name: name.clone(),
                rule_description: Some(String::new()),
                rule_configuration: config,
            };
            let rule = client.create_waf_rule(body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = WafRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldWafAction::UpdateRule { id, name } => {
            if name.is_none() {
                bail!("at least one update flag is required (--name)");
            }
            // The Shield API requires all fields on PATCH, so fetch current state first.
            let current = client.get_waf_rule(*id).await?;
            let body = UpdateCustomWafRule {
                rule_name: if name.is_some() {
                    name.clone()
                } else {
                    current.rule_name
                },
                rule_description: current.rule_description.or(Some(String::new())),
                rule_configuration: current.rule_configuration,
            };
            let rule = client.update_waf_rule(*id, body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = WafRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldWafAction::DeleteRule { id } => {
            if !yes {
                eprint!("Delete WAF rule {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_waf_rule(*id).await?;
            eprintln!("Deleted WAF rule {id}");
        }
        ShieldWafAction::TriggeredRules { shield_zone_id } => {
            let result = client.get_triggered_waf_rules(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let rules = result.triggered_rules.unwrap_or_default();
                let rows: Vec<TriggeredRuleRow> =
                    rules.iter().map(TriggeredRuleRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldWafAction::ReviewTriggeredRule {
            shield_zone_id,
            rule_id,
            action,
        } => {
            let body = UpdateReviewTriggeredRuleRequest {
                rule_id: Some(rule_id.clone()),
                action: u8_to_enum::<ReviewActionType>(*action, "action")?,
            };
            let result = client
                .review_triggered_waf_rule(*shield_zone_id, body)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let success = result.success.unwrap_or(false);
                eprintln!(
                    "Reviewed WAF rule {rule_id}: {}",
                    if success { "success" } else { "failed" }
                );
            }
        }
        ShieldWafAction::RecommendTriggeredRule {
            shield_zone_id,
            rule_id,
        } => {
            let result = client
                .get_triggered_waf_rule_recommendation(*shield_zone_id, rule_id)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(rec) = &result.recommendation {
                println!("Recommendation for rule {rule_id}:\n{rec}");
            } else {
                eprintln!("No recommendation available for rule {rule_id}.");
            }
        }
        ShieldWafAction::PlanSegmentation => {
            let result = client.get_waf_plan_segmentation().await?;
            let json =
                serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
            println!("{json}");
        }
        ShieldWafAction::EngineConfig => {
            let result = client.get_waf_engine_config().await?;
            let json =
                serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
            println!("{json}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Rate limit handler
// ---------------------------------------------------------------------------

async fn handle_rate_limit(
    action: &ShieldRateLimitAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldRateLimitAction::List { shield_zone_id } => {
            let rules = client.list_rate_limit_rules(*shield_zone_id).await?;
            let rows: Vec<RateLimitRuleRow> = rules.iter().map(RateLimitRuleRow::from).collect();
            output::print_data(&rows, format);
        }
        ShieldRateLimitAction::Get { id } => {
            let rule = client.get_rate_limit_rule(*id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = RateLimitRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldRateLimitAction::Create {
            shield_zone_id,
            name,
            action_type,
            operator_type,
            severity_type,
            value,
            request_count,
            counter_key_type,
            timeframe,
            block_time,
        } => {
            let config = RateLimitRuleConfiguration {
                action_type: u8_to_enum(*action_type, "action-type")?,
                variable_types: Some(Default::default()),
                operator_type: u8_to_enum(*operator_type, "operator-type")?,
                severity_type: u8_to_enum(*severity_type, "severity-type")?,
                transformation_types: Some(vec![]),
                value: value.clone(),
                request_count: *request_count,
                counter_key_type: u8_to_enum::<RateLimitCounterKey>(
                    *counter_key_type,
                    "counter-key-type",
                )?,
                timeframe: u16_to_enum(*timeframe, "timeframe")?,
                block_time: u16_to_enum(*block_time, "block-time")?,
                chained_rule_conditions: None,
            };
            let body = CreateRateLimitRule {
                shield_zone_id: *shield_zone_id,
                rule_name: name.clone(),
                rule_description: Some(String::new()),
                rule_configuration: config,
            };
            let rule = client.create_rate_limit_rule(body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = RateLimitRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldRateLimitAction::Update { id, name } => {
            if name.is_none() {
                bail!("at least one update flag is required (--name)");
            }
            // The Shield API requires all fields on PATCH, so fetch current state first.
            let current = client.get_rate_limit_rule(*id).await?;
            let body = UpdateRateLimitRule {
                rule_name: if name.is_some() {
                    name.clone()
                } else {
                    current.rule_name
                },
                rule_description: current.rule_description.or(Some(String::new())),
                rule_configuration: current.rule_configuration,
            };
            let rule = client.update_rate_limit_rule(*id, body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&rule).expect("failed to serialize to JSON")
                );
            } else {
                let row = RateLimitRuleRow::from(&rule);
                output::print_single(&row, format);
            }
        }
        ShieldRateLimitAction::Delete { id } => {
            if !yes {
                eprint!("Delete rate limit rule {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_rate_limit_rule(*id).await?;
            eprintln!("Deleted rate limit rule {id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Access list handler
// ---------------------------------------------------------------------------

async fn handle_access_list(
    action: &ShieldAccessListAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldAccessListAction::List { shield_zone_id } => {
            let result = client.get_access_lists(*shield_zone_id).await?;
            let managed = result.managed_lists.unwrap_or_default();
            let custom = result.custom_lists.unwrap_or_default();
            let all: Vec<&AccessListDetails> = managed.iter().chain(custom.iter()).collect();
            let rows: Vec<AccessListRow> = all.iter().map(|l| AccessListRow::from(*l)).collect();
            output::print_data(&rows, format);
        }
        ShieldAccessListAction::Get { shield_zone_id, id } => {
            let list = client.get_custom_access_list(*shield_zone_id, *id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&list).expect("failed to serialize to JSON")
                );
            } else {
                let row = CustomAccessListRow::from(&list);
                output::print_single(&row, format);
            }
        }
        ShieldAccessListAction::Create {
            shield_zone_id,
            name,
            r#type,
            content,
        } => {
            let body = CreateCustomAccessList {
                name: name.clone(),
                description: None,
                list_type: u8_to_enum::<AccessListType>(*r#type, "list-type")?,
                content: content.clone(),
                checksum: None,
            };
            let list = client.create_access_list(*shield_zone_id, body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&list).expect("failed to serialize to JSON")
                );
            } else {
                let row = CustomAccessListRow::from(&list);
                output::print_single(&row, format);
            }
        }
        ShieldAccessListAction::Update {
            shield_zone_id,
            id,
            name,
            content,
        } => {
            if name.is_none() && content.is_none() {
                bail!("at least one update flag is required (--name, --content)");
            }
            let body = UpdateCustomAccessList {
                name: name.clone(),
                content: content.clone(),
                checksum: None,
            };
            let list = client
                .update_custom_access_list(*shield_zone_id, *id, body)
                .await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&list).expect("failed to serialize to JSON")
                );
            } else {
                let row = CustomAccessListRow::from(&list);
                output::print_single(&row, format);
            }
        }
        ShieldAccessListAction::Delete { shield_zone_id, id } => {
            if !yes {
                eprint!("Delete access list {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                if !is_confirmed(&line) {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_access_list(*shield_zone_id, *id).await?;
            eprintln!("Deleted access list {id}");
        }
        ShieldAccessListAction::UpdateConfig {
            shield_zone_id,
            configuration_id,
            is_enabled,
            action,
        } => {
            if is_enabled.is_none() && action.is_none() {
                bail!("at least one update flag is required (--is-enabled, --action)");
            }
            let body = UpdateAccessListConfiguration {
                is_enabled: *is_enabled,
                action: action
                    .map(|v| u8_to_enum::<AccessListAction>(v, "action"))
                    .transpose()?,
            };
            client
                .update_access_list_configuration(*shield_zone_id, *configuration_id, body)
                .await?;
            eprintln!("Updated access list configuration {configuration_id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Bot detection handler
// ---------------------------------------------------------------------------

async fn handle_bot_detection(
    action: &ShieldBotDetectionAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldBotDetectionAction::Get { shield_zone_id } => {
            let result = client.get_bot_detection(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                match &result.data {
                    Some(state) => println!(
                        "{}",
                        serde_json::to_string_pretty(state).expect("failed to serialize to JSON")
                    ),
                    None => eprintln!("No bot detection data returned."),
                }
            } else if let Some(state) = &result.data {
                let row = BotDetectionRow::from(state);
                output::print_single(&row, format);
            } else {
                eprintln!("No bot detection data returned.");
            }
        }
        ShieldBotDetectionAction::Update {
            shield_zone_id,
            execution_mode,
            request_integrity_sensitivity,
            ip_address_sensitivity,
            fingerprint_sensitivity,
            fingerprint_aggression,
            fingerprint_complex_enabled,
        } => {
            if execution_mode.is_none()
                && request_integrity_sensitivity.is_none()
                && ip_address_sensitivity.is_none()
                && fingerprint_sensitivity.is_none()
                && fingerprint_aggression.is_none()
                && fingerprint_complex_enabled.is_none()
            {
                bail!(
                    "at least one update flag is required (--execution-mode, --request-integrity-sensitivity, --ip-address-sensitivity, --fingerprint-sensitivity, --fingerprint-aggression, --fingerprint-complex-enabled)"
                );
            }

            let request_integrity = request_integrity_sensitivity
                .map(|v| {
                    u8_to_enum::<BotDetectionSensitivity>(v, "request-integrity-sensitivity").map(
                        |s| RequestIntegrityConfiguration {
                            sensitivity: Some(s),
                        },
                    )
                })
                .transpose()?;

            let ip_address = ip_address_sensitivity
                .map(|v| {
                    u8_to_enum::<BotDetectionSensitivity>(v, "ip-address-sensitivity").map(|s| {
                        IpAddressConfiguration {
                            sensitivity: Some(s),
                        }
                    })
                })
                .transpose()?;

            let fingerprint_sens = fingerprint_sensitivity
                .map(|v| u8_to_enum::<BotDetectionSensitivity>(v, "fingerprint-sensitivity"))
                .transpose()?;
            let fingerprint_aggr = fingerprint_aggression
                .map(|v| u8_to_enum::<BrowserFingerprintAggression>(v, "fingerprint-aggression"))
                .transpose()?;

            let browser_fingerprint = if fingerprint_sens.is_some()
                || fingerprint_aggr.is_some()
                || fingerprint_complex_enabled.is_some()
            {
                Some(BrowserFingerprintConfiguration {
                    sensitivity: fingerprint_sens,
                    aggression: fingerprint_aggr,
                    complex_enabled: *fingerprint_complex_enabled,
                })
            } else {
                None
            };

            let body = UpdateBotDetection {
                shield_zone_id: *shield_zone_id,
                execution_mode: execution_mode
                    .map(|v| u8_to_enum::<BotDetectionExecutionMode>(v, "execution-mode"))
                    .transpose()?,
                request_integrity,
                ip_address,
                browser_fingerprint,
            };
            let result = client.update_bot_detection(*shield_zone_id, body).await?;
            if let OutputFormat::Json = format {
                match &result.data {
                    Some(state) => println!(
                        "{}",
                        serde_json::to_string_pretty(state).expect("failed to serialize to JSON")
                    ),
                    None => eprintln!("Updated bot detection for Shield Zone {shield_zone_id}"),
                }
            } else if let Some(state) = &result.data {
                let row = BotDetectionRow::from(state);
                output::print_single(&row, format);
            } else {
                eprintln!("Updated bot detection for Shield Zone {shield_zone_id}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Metrics handler
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct MetricsRow {
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Count")]
    count: i64,
}

async fn handle_metrics(
    action: &ShieldMetricsAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;
    match action {
        ShieldMetricsAction::Overview { shield_zone_id } => {
            let metrics = client.get_metrics_overview(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                if let Some(overview) = &data.overview {
                    let rows = vec![
                        MetricsRow {
                            category: "DDoS Mitigated".to_string(),
                            count: overview.d_do_s_mitigated,
                        },
                        MetricsRow {
                            category: "WAF Triggered".to_string(),
                            count: overview.waf_triggered_rules,
                        },
                        MetricsRow {
                            category: "Rate Limit Breaches".to_string(),
                            count: overview.ratelimit_breaches,
                        },
                        MetricsRow {
                            category: "Bot Detection Challenged".to_string(),
                            count: overview.bot_detection_challenged,
                        },
                        MetricsRow {
                            category: "Access List Actions".to_string(),
                            count: overview.access_list_actions,
                        },
                        MetricsRow {
                            category: "Upload Scanning Blocks".to_string(),
                            count: overview.upload_scanning_blocks,
                        },
                    ];
                    output::print_data(&rows, format);
                }
                if let Some(billable) = data.total_billable_requests {
                    eprintln!("Total billable requests: {billable}");
                }
            } else {
                eprintln!("No metrics data available.");
            }
        }
        ShieldMetricsAction::Detailed { shield_zone_id } => {
            let metrics = client.get_metrics_detailed(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                let mut rows = Vec::new();
                if let Some(waf) = &data.waf
                    && let Some(totals) = &waf.totals
                {
                    rows.push(MetricsRow {
                        category: "WAF Blocked".to_string(),
                        count: totals.blocked_requests,
                    });
                    rows.push(MetricsRow {
                        category: "WAF Logged".to_string(),
                        count: totals.logged_requests,
                    });
                    rows.push(MetricsRow {
                        category: "WAF Challenged".to_string(),
                        count: totals.challenged_requests,
                    });
                }
                if let Some(ddos) = &data.ddos
                    && let Some(totals) = &ddos.totals
                {
                    rows.push(MetricsRow {
                        category: "DDoS Blocked".to_string(),
                        count: totals.blocked_requests,
                    });
                    rows.push(MetricsRow {
                        category: "DDoS Verified".to_string(),
                        count: totals.verified_requests,
                    });
                    rows.push(MetricsRow {
                        category: "DDoS Challenged".to_string(),
                        count: totals.challenged_requests,
                    });
                }
                if let Some(rl) = &data.rate_limit
                    && let Some(totals) = &rl.totals
                {
                    rows.push(MetricsRow {
                        category: "Rate Limit Breaches".to_string(),
                        count: totals.total_breaches,
                    });
                    rows.push(MetricsRow {
                        category: "Rate Limit Blocked".to_string(),
                        count: totals.blocked_breaches,
                    });
                }
                if let Some(al) = &data.access_lists
                    && let Some(totals) = &al.totals
                {
                    rows.push(MetricsRow {
                        category: "Access List Blocked".to_string(),
                        count: totals.blocked_requests,
                    });
                }
                if let Some(bd) = &data.bot_detection
                    && let Some(totals) = &bd.totals
                {
                    rows.push(MetricsRow {
                        category: "Bot Detection Challenged".to_string(),
                        count: totals.challenged_requests,
                    });
                }
                if let Some(us) = &data.upload_scanning
                    && let Some(totals) = &us.totals
                {
                    rows.push(MetricsRow {
                        category: "Upload Scanning Blocked".to_string(),
                        count: totals.blocked_requests,
                    });
                    rows.push(MetricsRow {
                        category: "Files Scanned".to_string(),
                        count: totals.files_scanned,
                    });
                }
                output::print_data(&rows, format);
                if let Some(billable) = data.total_billable_requests_this_month {
                    eprintln!("Total billable requests this month: {billable}");
                }
            } else {
                eprintln!("No detailed metrics data available.");
            }
        }
        ShieldMetricsAction::RateLimits { shield_zone_id } => {
            let metrics = client.get_metrics_rate_limits(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                for entry in data {
                    let label = entry
                        .ratelimit_id
                        .map_or_else(|| "unknown".to_string(), |id| id.to_string());
                    if let Some(overview) = &entry.overview {
                        let rows = vec![
                            MetricsRow {
                                category: "Total Breaches".to_string(),
                                count: overview.total_breaches,
                            },
                            MetricsRow {
                                category: "Blocked".to_string(),
                                count: overview.blocked_breaches,
                            },
                            MetricsRow {
                                category: "Logged".to_string(),
                                count: overview.logged_breaches,
                            },
                            MetricsRow {
                                category: "Challenged".to_string(),
                                count: overview.challenged_breaches,
                            },
                        ];
                        eprintln!("Rate limit rule {label}:");
                        output::print_data(&rows, format);
                    }
                }
            } else {
                eprintln!("No rate limit metrics data available.");
            }
        }
        ShieldMetricsAction::RateLimit { id } => {
            let metrics = client.get_metrics_rate_limit(*id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                if let Some(overview) = &data.overview {
                    let rows = vec![
                        MetricsRow {
                            category: "Total Breaches".to_string(),
                            count: overview.total_breaches,
                        },
                        MetricsRow {
                            category: "Blocked".to_string(),
                            count: overview.blocked_breaches,
                        },
                        MetricsRow {
                            category: "Logged".to_string(),
                            count: overview.logged_breaches,
                        },
                        MetricsRow {
                            category: "Challenged".to_string(),
                            count: overview.challenged_breaches,
                        },
                    ];
                    output::print_data(&rows, format);
                }
            } else {
                eprintln!("No rate limit metrics data available.");
            }
        }
        ShieldMetricsAction::WafRule {
            shield_zone_id,
            rule_id,
        } => {
            let metrics = client
                .get_metrics_waf_rule(*shield_zone_id, *rule_id)
                .await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                let rows = vec![
                    MetricsRow {
                        category: "Total Triggers".to_string(),
                        count: data.total_triggers,
                    },
                    MetricsRow {
                        category: "Blocked".to_string(),
                        count: data.blocked_requests,
                    },
                    MetricsRow {
                        category: "Logged".to_string(),
                        count: data.logged_requests,
                    },
                    MetricsRow {
                        category: "Challenged".to_string(),
                        count: data.challenged_requests,
                    },
                ];
                output::print_data(&rows, format);
            } else {
                eprintln!("No WAF rule metrics data available.");
            }
        }
        ShieldMetricsAction::BotDetection { shield_zone_id } => {
            let metrics = client.get_metrics_bot_detection(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                let rows = vec![
                    MetricsRow {
                        category: "Total Logged".to_string(),
                        count: data.total_logged_requests,
                    },
                    MetricsRow {
                        category: "Total Challenged".to_string(),
                        count: data.total_challenged_requests,
                    },
                ];
                output::print_data(&rows, format);
            } else {
                eprintln!("No bot detection metrics data available.");
            }
        }
        ShieldMetricsAction::UploadScanning { shield_zone_id } => {
            let metrics = client.get_metrics_upload_scanning(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&metrics)
                    .context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(data) = &metrics.data {
                let rows = vec![
                    MetricsRow {
                        category: "Total Logged".to_string(),
                        count: data.total_logged_requests,
                    },
                    MetricsRow {
                        category: "Total Blocked".to_string(),
                        count: data.total_blocked_requests,
                    },
                    MetricsRow {
                        category: "Files Scanned".to_string(),
                        count: data.total_files_scanned,
                    },
                ];
                output::print_data(&rows, format);
            } else {
                eprintln!("No upload scanning metrics data available.");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// API Guardian handler
// ---------------------------------------------------------------------------

async fn handle_api_guardian(
    action: &ShieldApiGuardianAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldApiGuardianAction::Get { shield_zone_id } => {
            let result = client.get_api_guardian(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let endpoints = result.data.and_then(|d| d.endpoints).unwrap_or_default();
                let rows: Vec<ApiGuardianEndpointRow> =
                    endpoints.iter().map(ApiGuardianEndpointRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldApiGuardianAction::Upload {
            shield_zone_id,
            spec_file,
            enforce_authorization,
        } => {
            let contents = std::fs::read_to_string(spec_file)
                .with_context(|| format!("failed to read spec file {}", spec_file.display()))?;
            let body = UploadOpenApiSpecificationRequest {
                content: Some(contents),
                enforce_authorisation_validation: *enforce_authorization,
            };
            let result = client
                .upload_api_guardian_spec(*shield_zone_id, body)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let endpoints = result.data.and_then(|d| d.endpoints).unwrap_or_default();
                let rows: Vec<ApiGuardianEndpointRow> =
                    endpoints.iter().map(ApiGuardianEndpointRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldApiGuardianAction::Update {
            shield_zone_id,
            spec_file,
            enforce_authorization,
        } => {
            let contents = std::fs::read_to_string(spec_file)
                .with_context(|| format!("failed to read spec file {}", spec_file.display()))?;
            let body = UpdateApiGuardianRequest {
                content: contents,
                enforce_authorisation_validation: *enforce_authorization,
            };
            let result = client.update_api_guardian(*shield_zone_id, body).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let endpoints = result.data.and_then(|d| d.endpoints).unwrap_or_default();
                let rows: Vec<ApiGuardianEndpointRow> =
                    endpoints.iter().map(ApiGuardianEndpointRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ShieldApiGuardianAction::UpdateEndpoint {
            shield_zone_id,
            endpoint_id,
            enabled,
            validate_request_body_schema,
            validate_response_body_schema,
            validate_authorization,
        } => {
            if enabled.is_none()
                && validate_request_body_schema.is_none()
                && validate_response_body_schema.is_none()
                && validate_authorization.is_none()
            {
                bail!(
                    "at least one update flag is required (--enabled, --validate-request-body-schema, --validate-response-body-schema, --validate-authorization)"
                );
            }
            let body = UpdateApiGuardianEndpointRequest {
                enabled: *enabled,
                validate_request_body_schema: *validate_request_body_schema,
                validate_response_body_schema: *validate_response_body_schema,
                validate_authorization: *validate_authorization,
            };
            let result = client
                .update_api_guardian_endpoint(*shield_zone_id, *endpoint_id, body)
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(endpoint) = &result.data {
                let row = ApiGuardianEndpointRow::from(endpoint);
                output::print_single(&row, format);
            } else {
                eprintln!("Updated API Guardian endpoint {endpoint_id}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Upload Scanning handler
// ---------------------------------------------------------------------------

async fn handle_upload_scanning(
    action: &ShieldUploadScanningAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;

    match action {
        ShieldUploadScanningAction::Get { shield_zone_id } => {
            let result = client.get_upload_scanning(*shield_zone_id).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(state) = &result.data {
                let row = UploadScanningRow::from(state);
                output::print_single(&row, format);
            } else {
                eprintln!("No upload scanning data returned.");
            }
        }
        ShieldUploadScanningAction::Update {
            shield_zone_id,
            enabled,
            antivirus_mode,
            csam_mode,
        } => {
            if enabled.is_none() && antivirus_mode.is_none() && csam_mode.is_none() {
                bail!(
                    "at least one update flag is required (--enabled, --antivirus-mode, --csam-mode)"
                );
            }
            let body = UpdateUploadScanningConfigurationRequest {
                shield_zone_id: i32::try_from(*shield_zone_id)
                    .context("shield-zone-id too large for i32")?,
                is_enabled: *enabled,
                antivirus_scanning_mode: antivirus_mode
                    .map(|v| u8_to_enum::<UploadScanningScannerMode>(v, "antivirus-mode"))
                    .transpose()?,
                csam_scanning_mode: csam_mode
                    .map(|v| u8_to_enum::<UploadScanningScannerMode>(v, "csam-mode"))
                    .transpose()?,
            };
            let result = client.update_upload_scanning(*shield_zone_id, body).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
                println!("{json}");
            } else if let Some(state) = &result.data {
                let row = UploadScanningRow::from(state);
                output::print_single(&row, format);
            } else {
                eprintln!("Updated upload scanning for Shield Zone {shield_zone_id}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Event Logs handler
// ---------------------------------------------------------------------------

async fn handle_event_logs(
    shield_zone_id: i64,
    date: &str,
    continuation_token: Option<&str>,
    all: bool,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;
    let mut token = continuation_token.unwrap_or("").to_owned();

    loop {
        let result = client.get_event_logs(shield_zone_id, date, &token).await?;

        if let OutputFormat::Json = format {
            let json =
                serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
            println!("{json}");
        } else {
            let logs = result.logs.as_deref().unwrap_or(&[]);
            if logs.is_empty() {
                eprintln!("No event logs returned.");
            } else {
                let rows: Vec<EventLogRow> = logs.iter().map(EventLogRow::from).collect();
                output::print_data(&rows, format);
            }
        }

        let has_more = result.has_more_data.unwrap_or(false);
        let next_token = result.continuation_token;

        if !all || !has_more {
            if has_more && let Some(ref t) = next_token {
                eprintln!(
                    "More data available. Use --continuation-token {t} to get the next page."
                );
            }
            break;
        }

        match next_token {
            Some(t) if !t.is_empty() => token = t,
            _ => break,
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Pullzone mapping handler
// ---------------------------------------------------------------------------

async fn handle_pullzone_mapping(
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::shield_client(debug, record)?;
    let result = client.get_shield_zones_pullzone_mapping().await?;
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?;
        println!("{json}");
    } else {
        let mappings = result.data.unwrap_or_default();
        let rows: Vec<PullzoneMappingRow> = mappings.iter().map(PullzoneMappingRow::from).collect();
        output::print_data(&rows, format);
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
