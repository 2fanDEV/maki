use anyhow::Result;
use aws_config::SdkConfig;
use aws_sdk_s3::{Client, client::Waiters, config::Region, types::MetadataConfiguration};
use env_logger::DEFAULT_FILTER_ENV;
use log::debug;
use regex::Regex;

pub struct EnsuredS3BucketOutput {
    pub location: String,
    pub name: String,
}

pub struct S3Util {
    config: SdkConfig,
    s3_client: Client,
}

impl S3Util {
    pub const DEFAULT_BUCKET: &str = "default";
    pub async fn new(region: &str) -> Result<Self> {
        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_owned()))
            .load()
            .await;
        let s3_client = Client::new(&config);

        Ok(Self { config, s3_client })
    }

    pub async fn upload(
        &self,
        bucket_name: Option<&str>,
        metadata: Map<String, Object>,
        obj: Vec<u8>,
    ) -> Result<()> {
        Ok(())
    }

    async fn ensure_bucket(&self, bucket_name: Option<&str>) -> EnsuredS3BucketOutput {
        let target_bucket = bucket_name.map_or(Self::DEFAULT_BUCKET, |bucket_name| bucket_name);
        let bucket_location = match self
            .s3_client
            .head_bucket()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(found) => found.bucket_location_name().unwrap(),
            Err(e) => self
                .s3_client
                .create_bucket()
                .bucket(target_name)
                .bucket_namespace(aws_sdk_s3::types::BucketNamespace::AccountRegional)
                .send()
                .await?
                .location()
                .unwrap(),
        };
        EnsuredS3BucketOutput {
            name: target_bucket.to_owned(),
            location: bucket_location.to_owned(),
        }
    }

    pub fn validate_bucket_namespaced_name(bucket_name: &str) -> bool {
        Regex::new("")
    }
}
