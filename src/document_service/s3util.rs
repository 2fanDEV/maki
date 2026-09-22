use std::collections::HashMap;

use anyhow::{Result, anyhow, bail};
use aws_config::SdkConfig;
use aws_credential_types::provider::ProvideCredentials;
use aws_sdk_s3::{
    Client,
    config::Region,
    primitives::ByteStream,
    types::{BucketLocationConstraint, BucketNamespace, CreateBucketConfiguration},
};

#[derive(Clone)]
pub(super) struct S3Util {
    account_id: String,
    region: String,
    s3_client: Client,
}

impl S3Util {
    pub(super) async fn new(
        configured_region: Option<&str>,
        configured_account_id: Option<&str>,
    ) -> Result<Self> {
        let mut config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest());
        if let Some(region) = configured_region {
            config_loader = config_loader.region(Region::new(region.to_owned()));
        }
        let config = config_loader.load().await;

        let region = resolve_region(
            configured_region,
            config.region().map(|region| region.as_ref()),
        )?;
        let account_id = match configured_account_id {
            Some(account_id) => resolve_account_id(Some(account_id), None)?,
            None => account_id_from_sdk(&config).await?,
        };
        let s3_client = Client::new(&config);

        Ok(Self {
            account_id,
            region,
            s3_client,
        })
    }

    pub(super) fn namespaced_bucket_name(&self, prefix: &str) -> Result<String> {
        namespaced_bucket_name(prefix, &self.account_id, &self.region)
    }

    pub(super) async fn upload(
        &self,
        bucket_name: &str,
        object_key: &str,
        content_type: &str,
        metadata: HashMap<String, String>,
        bytes: Vec<u8>,
    ) -> Result<Option<String>> {
        self.ensure_bucket(bucket_name).await?;
        let output = self
            .s3_client
            .put_object()
            .bucket(bucket_name)
            .key(object_key)
            .content_type(content_type)
            .set_metadata(Some(metadata))
            .body(ByteStream::from(bytes))
            .send()
            .await?;

        Ok(output.e_tag().map(str::to_owned))
    }

    async fn ensure_bucket(&self, bucket_name: &str) -> Result<()> {
        match self
            .s3_client
            .head_bucket()
            .bucket(bucket_name)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|error| error.is_not_found()) =>
            {
                self.create_bucket(bucket_name).await
            }
            Err(error) => Err(error.into()),
        }
    }

    async fn create_bucket(&self, bucket_name: &str) -> Result<()> {
        let mut request = self
            .s3_client
            .create_bucket()
            .bucket(bucket_name)
            .bucket_namespace(BucketNamespace::AccountRegional);

        if self.region != "us-east-1" {
            request = request.create_bucket_configuration(
                CreateBucketConfiguration::builder()
                    .location_constraint(BucketLocationConstraint::from(self.region.as_str()))
                    .build(),
            );
        }

        match request.send().await {
            Ok(_) => Ok(()),
            Err(error)
                if error
                    .as_service_error()
                    .is_some_and(|error| error.is_bucket_already_owned_by_you()) =>
            {
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }
}

async fn account_id_from_sdk(config: &SdkConfig) -> Result<String> {
    let credentials = config
        .credentials_provider()
        .ok_or_else(|| {
            anyhow!(
                "AWS account ID is required; set aws.account_id or provide it through the AWS SDK credential chain (for example AWS_ACCOUNT_ID)"
            )
        })?
        .provide_credentials()
        .await
        .map_err(|error| {
            anyhow!(
                "AWS account ID is required; set aws.account_id or provide it through the AWS SDK credential chain (for example AWS_ACCOUNT_ID); credential resolution failed: {error}"
            )
        })?;
    resolve_account_id(
        None,
        credentials
            .account_id()
            .map(|account_id| account_id.as_str()),
    )
}

pub(super) fn resolve_region(configured: Option<&str>, sdk_region: Option<&str>) -> Result<String> {
    let region = configured
        .or(sdk_region)
        .map(str::trim)
        .filter(|region| !region.is_empty())
        .ok_or_else(|| {
            anyhow!(
                "AWS region is required; set aws.region or configure the AWS SDK region chain (for example AWS_REGION or AWS_DEFAULT_REGION)"
            )
        })?;

    if !valid_name_component(region) {
        bail!("AWS region must contain only lowercase ASCII letters, digits, and hyphens");
    }
    Ok(region.to_owned())
}

pub(super) fn resolve_account_id(
    configured: Option<&str>,
    sdk_account_id: Option<&str>,
) -> Result<String> {
    let account_id = configured
        .or(sdk_account_id)
        .map(str::trim)
        .filter(|account_id| !account_id.is_empty())
        .ok_or_else(|| {
            anyhow!(
                "AWS account ID is required; set aws.account_id or provide it through the AWS SDK credential chain (for example AWS_ACCOUNT_ID)"
            )
        })?;

    if account_id.len() != 12 || !account_id.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("AWS account ID must contain exactly 12 digits");
    }
    Ok(account_id.to_owned())
}

pub(super) fn namespaced_bucket_name(
    prefix: &str,
    account_id: &str,
    region: &str,
) -> Result<String> {
    let prefix = prefix.trim();
    if !valid_bucket_prefix(prefix) {
        bail!(
            "bucket prefix must start and end with a lowercase ASCII letter or digit and contain only lowercase ASCII letters, digits, and hyphens"
        );
    }
    if ["xn--", "sthree-", "amzn-s3-demo-"]
        .iter()
        .any(|reserved| prefix.starts_with(reserved))
    {
        bail!("bucket prefix starts with an AWS-reserved prefix");
    }

    let bucket_name = format!("{prefix}-{account_id}-{region}-an");
    if bucket_name.len() > 63 {
        bail!("account-regional bucket name exceeds the 63-character S3 limit");
    }
    Ok(bucket_name)
}

fn valid_bucket_prefix(prefix: &str) -> bool {
    !prefix.is_empty()
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && prefix
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && prefix
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}

fn valid_name_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
}
