use std::iter::Map;

use anyhow::Result;
use serde_json::Value;

use crate::document_service::s3util::S3Util;

pub mod api;
mod s3util;
pub mod types;

#[derive(Clone)]
pub struct DocumentService {
    s3_util: S3Util,
}

impl DocumentService {
    pub const DEFAULT_BUCKET: &str = "DEFAULT";
    pub async fn new(region: Option<&str>) -> Result<Self> {
        Ok(Self {
            s3_util: S3Util::new(region).await?,
        })
    }

    async fn upload_to_s3(
        &mut self,
        metadata: Map<String, Value>,
        bucket_name: Option<&str>,
        document: Vec<u8>,
    ) -> Result<()> {
        let bucket = self.s3_util.ensure_bucket(bucket_name).await?;
        Ok(())
    }
}
