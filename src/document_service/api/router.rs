use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::{
        Multipart, State,
        multipart::{self, MultipartRejection},
    },
    http::StatusCode,
};
use axum_typed_multipart::TypedMultipart;

use super::{request::UploadDocumentRequest, response::UploadDocumentResponse};
use crate::{
    api::response::ApiError, document_service::DocumentService, label_service::LabelService,
};

/// Upload a document to an account-regional S3 bucket.
#[utoipa::path(
    post,
    path = "/",
    tag = "documents",
    operation_id = "uploadDocument",
    request_body(content = UploadDocumentRequest, content_type = "multipart/form-data"),
    responses(
        (status = 201, body = UploadDocumentResponse),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 413, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn upload_document(
    State(document_service): State<DocumentService>,
    Extension(_label_service): Extension<LabelService>,
    multipart: TypedMultipart<UploadDocumentRequest>,
) -> Result<(StatusCode, Json<UploadDocumentResponse>), ApiError> {
    let metadata: HashMap<String, String> =
        multipart
            .metadata
            .clone()
            .map_or(HashMap::new(), |json_str| {
                serde_json::from_str::<HashMap<String, String>>(&json_str)
                    .map_err(|err| err)
                    .unwrap()
            });
    let file_metadata = multipart.file.metadata.clone();
    let uploaded = document_service
        .upload(
            &file_metadata
                .file_name
                .expect("File name must not be empty"),
            &multipart.file.contents,
            &file_metadata
                .content_type
                .expect("Content-Type must not be empty"),
            multipart.bucket_name.as_deref(),
            metadata,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(uploaded.into())))
}

fn reject_duplicate(duplicate: bool, field: &str) -> Result<(), ApiError> {
    if duplicate {
        Err(ApiError(
            StatusCode::BAD_REQUEST,
            format!("duplicate multipart field: {field}"),
        ))
    } else {
        Ok(())
    }
}
