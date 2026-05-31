use crate::auth;
use crate::cli::OutputFormat;
use crate::output;
use anyhow::Result;

pub async fn handle(
    url: &str,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(debug, record)?;
    client.purge_url(url).await?;
    output::print_mutation_result(
        format,
        "purge",
        "cache",
        serde_json::json!({ "Url": url }),
        &format!("Purged {url}"),
    );
    Ok(())
}
