use anyhow::Result;
use config::{Config, File, FileFormat};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub s3: S3Config,
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

#[derive(Deserialize)]
pub struct S3Config {
    pub region: String,
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
