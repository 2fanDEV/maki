use std::collections::HashMap;

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::{config::AwsConfig, document_service::s3util::S3Util, service::ServiceError};

pub mod api;
mod s3util;
#[cfg(test)]
mod tests;
pub mod types;

#[derive(Clone)]
pub struct DocumentService {
    default_bucket: String,
    s3_util: S3Util,
}

pub(super) struct UploadedDocument {
    pub bucket_name: String,
    pub object_key: String,
    pub e_tag: Option<String>,
}

impl DocumentService {
    pub const DEFAULT_BUCKET: &str = "default";

    pub async fn new(config: &AwsConfig) -> Result<Self> {
        let s3_util = S3Util::new(config.region.as_deref(), config.account_id.as_deref()).await?;
        let default_bucket = default_bucket_prefix(config.default_bucket.as_deref()).to_owned();
        s3_util
            .namespaced_bucket_name(&default_bucket)
            .context("invalid default S3 bucket prefix")?;

        Ok(Self {
            default_bucket,
            s3_util,
        })
    }

    pub(super) async fn upload(
        &self,
        file_name: &str,
        bytes: Vec<u8>,
        content_type: &str,
        bucket_prefix: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<UploadedDocument, ServiceError> {
        let file_name = safe_file_name(file_name)?;
        let prefix = selected_bucket_prefix(bucket_prefix, &self.default_bucket);
        let bucket_name = self
            .s3_util
            .namespaced_bucket_name(prefix)
            .map_err(|error| ServiceError::InvalidInput(error.to_string()))?;
        let object_key = object_key(&file_name);
        let metadata = upload_metadata(metadata, &bucket_name);
        let e_tag = self
            .s3_util
            .upload(&bucket_name, &object_key, content_type, metadata, bytes)
            .await
            .map_err(ServiceError::Internal)?;

        Ok(UploadedDocument {
            bucket_name,
            object_key,
            e_tag,
        })
    }
}

fn default_bucket_prefix(configured: Option<&str>) -> &str {
    configured.unwrap_or(DocumentService::DEFAULT_BUCKET)
}

fn selected_bucket_prefix<'a>(requested: Option<&'a str>, default: &'a str) -> &'a str {
    requested.unwrap_or(default)
}

fn safe_file_name(file_name: &str) -> Result<String, ServiceError> {
    let file_name = file_name
        .rsplit(['/', '\\'])
        .next()
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .ok_or_else(|| ServiceError::InvalidInput("file name is invalid".into()))?;
    Ok(file_name.to_owned())
}

fn object_key(file_name: &str) -> String {
    format!("{}/{file_name}", Uuid::new_v4())
}

fn upload_metadata(
    mut metadata: HashMap<String, String>,
    bucket_name: &str,
) -> HashMap<String, String> {
    metadata.insert("bucket_name".into(), bucket_name.to_owned());
    metadata
}
