use crate::auth;
use crate::cli::OutputFormat;
use crate::output;
use anyhow::Result;

pub async fn handle(
    url: &str,
    exact_path: bool,
    is_async: bool,
    format: OutputFormat,
    debug: bool,
    dry_run: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(&auth::ClientOpts {
        debug,
        dry_run,
        record,
        ..Default::default()
    })?;
    client.purge_url(url, exact_path, is_async).await?;
    output::print_mutation_result(
        format,
        "purge",
        "cache",
        serde_json::json!({ "Url": url, "ExactPath": exact_path, "Async": is_async }),
        &format!("Purged {url}"),
    );
    Ok(())
}
