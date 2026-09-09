use anyhow::{Context, Result, anyhow, ensure};
use mongodb::{Client, Database};

/// Initialize the shared handle without issuing a ping or application query.
pub async fn from_env() -> Result<Database> {
    let uri = std::env::var("MONGODB_URI").context("MONGODB_URI must be set")?;
    let name = std::env::var("MONGODB_DATABASE").context("MONGODB_DATABASE must be set")?;
    ensure!(!uri.trim().is_empty(), "MONGODB_URI must not be empty");
    ensure!(
        !name.trim().is_empty()
            && name.len() < 64
            && !name
                .chars()
                .any(|ch| "/\\.\"$*<>:|?\0".contains(ch) || ch.is_whitespace()),
        "MONGODB_DATABASE must be a valid database name"
    );
    // Driver errors may contain connection-string details; do not expose credentials.
    let client = Client::with_uri_str(&uri)
        .await
        .map_err(|_| anyhow!("could not initialize MongoDB client; check MONGODB_URI"))?;
    Ok(client.database(&name))
}
