use std::collections::HashMap;

use super::{
    DocumentService, default_bucket_prefix, object_key,
    s3util::{namespaced_bucket_name, resolve_account_id, resolve_region},
    safe_file_name, selected_bucket_prefix, upload_metadata,
};

#[test]
fn selects_requested_configured_and_constant_bucket_prefixes() {
    assert_eq!(
        selected_bucket_prefix(Some("invoices"), "configured"),
        "invoices"
    );
    assert_eq!(default_bucket_prefix(Some("configured")), "configured");
    assert_eq!(default_bucket_prefix(None), DocumentService::DEFAULT_BUCKET);
}

#[test]
fn configured_aws_identity_takes_precedence_over_sdk_values() {
    assert_eq!(
        resolve_account_id(Some("123456789012"), Some("999999999999")).unwrap(),
        "123456789012"
    );
    assert_eq!(
        resolve_region(Some("eu-west-1"), Some("us-east-1")).unwrap(),
        "eu-west-1"
    );
}

#[test]
fn sdk_aws_identity_is_used_when_configuration_is_absent() {
    assert_eq!(
        resolve_account_id(None, Some("123456789012")).unwrap(),
        "123456789012"
    );
    assert_eq!(
        resolve_region(None, Some("eu-west-1")).unwrap(),
        "eu-west-1"
    );
}

#[test]
fn missing_aws_identity_has_actionable_errors() {
    assert!(
        resolve_account_id(None, None)
            .unwrap_err()
            .to_string()
            .contains("aws.account_id")
    );
    assert!(
        resolve_region(None, None)
            .unwrap_err()
            .to_string()
            .contains("aws.region")
    );
}

#[test]
fn constructs_account_regional_bucket_names() {
    assert_eq!(
        namespaced_bucket_name("default", "123456789012", "eu-west-1").unwrap(),
        "default-123456789012-eu-west-1-an"
    );
    assert_eq!(
        namespaced_bucket_name("customer-docs", "123456789012", "us-east-1").unwrap(),
        "customer-docs-123456789012-us-east-1-an"
    );
}

#[test]
fn rejects_invalid_bucket_prefixes() {
    for prefix in ["", "UPPERCASE", "-leading", "trailing-", "has.period"] {
        assert!(
            namespaced_bucket_name(prefix, "123456789012", "eu-west-1").is_err(),
            "accepted invalid prefix {prefix:?}"
        );
    }
}

#[test]
fn bucket_name_metadata_is_server_authoritative() {
    let mut metadata = HashMap::from([
        ("source".into(), "frontend".into()),
        ("bucket_name".into(), "untrusted".into()),
    ]);

    metadata = upload_metadata(metadata, "default-123456789012-eu-west-1-an");

    assert_eq!(metadata["source"], "frontend");
    assert_eq!(metadata["bucket_name"], "default-123456789012-eu-west-1-an");
}

#[test]
fn object_keys_are_unique_and_use_safe_file_names() {
    assert_eq!(safe_file_name("/tmp/report.pdf").unwrap(), "report.pdf");
    assert_eq!(
        safe_file_name(r"C:\fakepath\report.pdf").unwrap(),
        "report.pdf"
    );
    assert!(safe_file_name("..").is_err());

    let first = object_key("report.pdf");
    let second = object_key("report.pdf");
    assert_ne!(first, second);
    assert!(first.ends_with("/report.pdf"));
    assert!(second.ends_with("/report.pdf"));
}
