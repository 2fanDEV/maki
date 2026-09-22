use std::collections::HashMap;

use axum::{
    Extension, Json,
    extract::{Multipart, State, multipart::MultipartRejection},
    http::StatusCode,
};

use super::{request::UploadDocumentRequest, response::UploadDocumentResponse};
use crate::{
    api::response::ApiError, document_service::DocumentService, label_service::LabelService,
};

/// Upload a document to an account-regional S3 bucket.
#[utoipa::path(
    post,
    path = "/",
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
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<(StatusCode, Json<UploadDocumentResponse>), ApiError> {
    let request = parse_request(multipart?).await?;
    let uploaded = document_service
        .upload(
            &request.file_name,
            request.file,
            &request.content_type,
            request.bucket_name.as_deref(),
            request.metadata.unwrap_or_default(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(uploaded.into())))
}

async fn parse_request(mut multipart: Multipart) -> Result<UploadDocumentRequest, ApiError> {
    let mut file = None;
    let mut file_name = None;
    let mut content_type = None;
    let mut bucket_name = None;
    let mut metadata = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name().map(str::to_owned).ok_or_else(|| {
            ApiError(
                StatusCode::BAD_REQUEST,
                "multipart field has no name".into(),
            )
        })?;

        match field_name.as_str() {
            "file" => {
                reject_duplicate(file.is_some(), "file")?;
                file_name = Some(
                    field
                        .file_name()
                        .filter(|name| !name.trim().is_empty())
                        .map(str::to_owned)
                        .ok_or_else(|| {
                            ApiError(StatusCode::BAD_REQUEST, "file name is required".into())
                        })?,
                );
                content_type = Some(
                    field
                        .content_type()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "application/octet-stream".into()),
                );
                file = Some(field.bytes().await?.to_vec());
            }
            "bucket_name" => {
                reject_duplicate(bucket_name.is_some(), "bucket_name")?;
                bucket_name = Some(field.text().await?);
            }
            "metadata" => {
                reject_duplicate(metadata.is_some(), "metadata")?;
                let value = field.text().await?;
                metadata = Some(
                    serde_json::from_str::<HashMap<String, String>>(&value).map_err(|error| {
                        ApiError(
                            StatusCode::BAD_REQUEST,
                            format!("metadata must be a JSON object with string values: {error}"),
                        )
                    })?,
                );
            }
            _ => {
                return Err(ApiError(
                    StatusCode::BAD_REQUEST,
                    format!("unknown multipart field: {field_name}"),
                ));
            }
        }
    }

    Ok(UploadDocumentRequest {
        file: file.ok_or_else(|| ApiError(StatusCode::BAD_REQUEST, "file is required".into()))?,
        bucket_name,
        metadata,
        file_name: file_name.expect("file bytes and file name are set together"),
        content_type: content_type.expect("file bytes and content type are set together"),
    })
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
