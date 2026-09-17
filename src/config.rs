use anyhow::Result;
use config::{Config, File, FileFormat};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    #[serde(default)]
    pub aws: AwsConfig,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub mongodb_uri: String,
    pub mongodb_database: String,
}

#[derive(Deserialize)]
pub struct LoggingConfig {
    pub rust_log: String,
    pub rust_log_style: String,
}

#[derive(Clone, Default, Deserialize)]
pub struct AwsConfig {
    pub account_id: Option<String>,
    pub region: Option<String>,
    pub default_bucket: Option<String>,
}

pub fn load() -> Result<AppConfig> {
    Config::builder()
        .set_default("logging.rust_log", "info")?
        .set_default("logging.rust_log_style", "auto")?
        .add_source(File::from(Path::new(".env.toml")).format(FileFormat::Toml))
        .build()?
        .try_deserialize()
        .map_err(Into::into)
}
