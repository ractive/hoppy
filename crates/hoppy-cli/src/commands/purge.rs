use crate::auth;
use anyhow::Result;

pub async fn handle(url: &str, debug: bool, record: Option<&str>) -> Result<()> {
    let client = auth::core_client(debug, record)?;
    client.purge_url(url).await?;
    eprintln!("Purged {url}");
    Ok(())
}
