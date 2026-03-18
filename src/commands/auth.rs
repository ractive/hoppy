use crate::auth;
use crate::cli::{AuthAction, OutputFormat};
use crate::output;
use anyhow::Result;
use bunny_api_core::client::CoreClient;
use bunny_api_core::types::BillingDetails;

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
            b.automatic_recharge_treshold, b.automatic_payment_amount
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

    let bandwidth_gb = b.monthly_bandwidth_used as f64 / 1_073_741_824.0;

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
            value: format!("{bandwidth_gb:.2} GB"),
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
) -> Result<()> {
    match action {
        AuthAction::Check => handle_check(format, debug).await,
    }
}

async fn handle_check(format: OutputFormat, debug: bool) -> Result<()> {
    let client = CoreClient::new(auth::get_api_key()?).with_debug(debug);
    let billing = client.get_billing().await?;

    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&billing).expect("failed to serialize to JSON")
            );
        }
        _ => {
            let rows = billing_to_rows(&billing);
            output::print_data(&rows, format);
        }
    }

    Ok(())
}
