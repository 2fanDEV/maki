use axum::extract::multipart::{MultipartError, MultipartRejection};
use serde::Serialize;

use super::super::UploadedDocument;
use crate::api::response::ApiError;

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct UploadDocumentResponse {
    pub bucket_name: String,
    pub object_key: String,
    #[schema(required = true)]
    pub e_tag: Option<String>,
}

impl From<UploadedDocument> for UploadDocumentResponse {
    fn from(upload: UploadedDocument) -> Self {
        Self {
            bucket_name: upload.bucket_name,
            object_key: upload.object_key,
            e_tag: upload.e_tag,
        }
    }
}

impl From<MultipartRejection> for ApiError {
    fn from(error: MultipartRejection) -> Self {
        Self(error.status(), error.body_text())
    }
}

impl From<MultipartError> for ApiError {
    fn from(error: MultipartError) -> Self {
        Self(error.status(), error.body_text())
    }
}
