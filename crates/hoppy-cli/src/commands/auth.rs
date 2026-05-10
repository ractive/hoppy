use crate::auth;
use crate::cli::{AuthAction, OutputFormat};
use crate::output;
use anyhow::{Context, Result};
use bunny_net_api::core::types::BillingDetails;

// ---------------------------------------------------------------------------
// Display row
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct AccountRow {
    #[tabled(rename = "Field")]
    field: String,
    #[tabled(rename = "Value")]
    value: String,
}

fn billing_to_rows(b: &BillingDetails) -> Vec<AccountRow> {
    let auto_pay = if b.automatic_recharge_enabled {
        format!(
            "enabled (threshold ${:.2}, amount ${:.2})",
            b.automatic_recharge_threshold, b.automatic_payment_amount
        )
    } else {
        "disabled".to_owned()
    };

    let payment_method = match (
        &b.automatic_payment_card_type,
        &b.automatic_payment_identifier,
    ) {
        (Some(card_type), Some(identifier)) => format!("{card_type} {identifier}"),
        (Some(card_type), None) => card_type.clone(),
        _ => "-".to_owned(),
    };

    let bandwidth_gb = b.monthly_bandwidth_used / 1_073_741_824;
    let bandwidth_remainder = (b.monthly_bandwidth_used % 1_073_741_824) * 100 / 1_073_741_824;

    vec![
        AccountRow {
            field: "API Key".to_owned(),
            value: "valid".to_owned(),
        },
        AccountRow {
            field: "Balance".to_owned(),
            value: format!("${:.4}", b.balance),
        },
        AccountRow {
            field: "This Month Charges".to_owned(),
            value: format!("${:.4}", b.this_month_charges),
        },
        AccountRow {
            field: "Billing Enabled".to_owned(),
            value: b.billing_enabled.to_string(),
        },
        AccountRow {
            field: "Auto Recharge".to_owned(),
            value: auto_pay,
        },
        AccountRow {
            field: "Payment Method".to_owned(),
            value: payment_method,
        },
        AccountRow {
            field: "Monthly Bandwidth".to_owned(),
            value: format!("{bandwidth_gb}.{bandwidth_remainder:02} GB"),
        },
    ]
}

// ---------------------------------------------------------------------------
// Top-level handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &AuthAction,
    format: OutputFormat,
    debug: bool,
    _yes: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        AuthAction::Check => handle_check(format, debug, record).await,
    }
}

async fn handle_check(format: OutputFormat, debug: bool, record: Option<&str>) -> Result<()> {
    let client = auth::core_client(debug, record)?;
    let billing = client.get_billing().await?;

    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&billing).context("failed to serialize to JSON")?
            );
        }
        _ => {
            let rows = billing_to_rows(&billing);
            output::print_data(&rows, format);
        }
    }

    Ok(())
}
