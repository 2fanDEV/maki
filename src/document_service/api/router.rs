use axum::extract::State;

use crate::document_service::DocumentService;

/// Placeholder: returns an empty response.
#[utoipa::path(get, path = "/", responses((status = 200)))]
pub(super) async fn get_documents(State(service): State<DocumentService>) {
    service.get_documents().await;
}

/// Placeholder: returns an empty response.
#[utoipa::path(post, path = "/", responses((status = 200)))]
pub(super) async fn classify_documents(State(service): State<DocumentService>) {
    service.classify_documents().await;
}

/// Placeholder: returns an empty response.
#[utoipa::path(options, path = "/", responses((status = 200)))]
pub(super) async fn query_documents(State(service): State<DocumentService>) {
    service.query_documents().await;
}
