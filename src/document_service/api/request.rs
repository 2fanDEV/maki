use std::collections::HashMap;

use axum::body::Bytes;
use axum_typed_multipart::{FieldData, TryFromField, TryFromMultipart};
use serde::Deserialize;
use uuid::Uuid;

#[derive(TryFromField, utoipa::ToSchema)]
pub enum UploadMethod {
    CREATE_BUCKET,
    APPEND,
}

#[derive(TryFromMultipart, utoipa::ToSchema)]
pub struct UploadDocumentRequest {
    #[schema(value_type = String, format = Binary)]
    pub(super) file: FieldData<Bytes>,
    pub(super) bucket_name: Option<String>,
    pub(super) method: UploadMethod,
    pub(super) metadata: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DocumentStack {
    pub name: String,
    pub documents: Vec<Uuid>,
}
