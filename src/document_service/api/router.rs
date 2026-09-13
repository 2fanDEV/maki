use axum::{Extension, Json, extract::State, http::StatusCode};

use crate::{
    api::response::ApiError,
    document_service::{DocumentService, api::request::UploadDocumentRequest},
    label_service::LabelService,
};

async fn upload_document(
    State(service): State<DocumentService>,
    Extension(service): Extension<LabelService>,
    Json(payload): Json<UploadDocumentRequest>,
) -> Result<StatusCode, ApiError> {
    Ok(StatusCode::CREATED)
}
