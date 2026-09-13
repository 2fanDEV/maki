//! Request types for the document API will live here when its endpoints are implemented.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UploadDocumentRequest {
    pub name: String,
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub metadata: serde_json::Value,
}

#[derive(Deserialize, utoipa::ToSchema)]#[serde(deny_unknown_fields)]
pub struct DocumentStack {
    pub name: String,
    pub documents: Vec<Uuid>,
}

}
