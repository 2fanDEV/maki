use std::iter::Map;

use anyhow::{Result, anyhow};
use aws_config::SdkConfig;
use aws_sdk_s3::{
    Client, config::Region, operation::create_bucket::CreateBucketOutput, types::Object,
};
use regex::Regex;

pub struct EnsuredS3BucketOutput {
    pub location: String,
    pub arn: String,
    pub name: String,
}

#[derive(Clone)]
pub struct S3Util {
    config: SdkConfig,
    s3_client: Client,
}

impl S3Util {
    pub const DEFAULT_BUCKET: &str = "default";
    pub async fn new(region: Option<&str>) -> Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(region) = region {
            config_loader = config_loader.region(Region::new(region.to_owned()));
        }
        let config = config_loader.load().await;
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

    pub async fn ensure_bucket(&self, bucket_name: Option<&str>) -> Result<EnsuredS3BucketOutput> {
        let target_bucket = bucket_name.map_or(Self::DEFAULT_BUCKET, |bucket_name| bucket_name);
        let map: fn(Option<&str>) -> String =
            |opt: Option<&str>| opt.map_or(String::from("undefined"), |s| s.to_string());
        let bucket_location = match self
            .s3_client
            .head_bucket()
            .bucket(target_bucket)
            .send()
            .await
        {
            Ok(found) => (map(found.bucket_location_name()), map(found.bucket_arn())),
            Err(e) => {
                log::error!("{:?}", e);
                let create_s3_bucket = { self.create_s3_bucket(target_bucket).await? };
                (
                    map(create_s3_bucket.location()),
                    map(create_s3_bucket.bucket_arn()),
                )
            }
        };
        Ok(EnsuredS3BucketOutput {
            name: target_bucket.to_owned(),
            location: bucket_location.0,
            arn: bucket_location.1,
        })
    }

    async fn create_s3_bucket(&self, bucket_name: &str) -> Result<CreateBucketOutput> {
        return match self
            .s3_client
            .create_bucket()
            .bucket(bucket_name)
            .bucket_namespace(aws_sdk_s3::types::BucketNamespace::AccountRegional)
            .send()
            .await
        {
            Ok(output) => Ok(output),
            Err(e) => Err(anyhow!(e)),
        };
    }

    pub fn validate_bucket_namespaced_name(bucket_name: &str) -> bool {
        let regex = Regex::new("^(?:[\\p{L}\\p{N}]+-){3}an$").unwrap();
        regex.is_match(bucket_name)
    }
}
