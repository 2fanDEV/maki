use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

#[derive(utoipa::ToSchema)]
pub struct UploadDocumentRequest {
    #[schema(value_type = String, format = Binary)]
    pub(super) file: Vec<u8>,
    pub(super) bucket_name: Option<String>,
    pub(super) metadata: Option<HashMap<String, String>>,
    #[schema(ignore)]
    pub(super) file_name: String,
    #[schema(ignore)]
    pub(super) content_type: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DocumentStack {
    pub name: String,
    pub documents: Vec<Uuid>,
}
