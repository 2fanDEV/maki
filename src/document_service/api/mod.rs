use aide::axum::{ApiRouter, routing::get_with};
use axum::extract::State;

use super::DocumentService;
use crate::service::Service;

pub mod request;
pub mod response;

impl DocumentService {
    pub const BASE_PATH: &str = "/documents";
}

impl Service for DocumentService {
    fn api_router(self) -> ApiRouter {
        ApiRouter::new()
            .api_route(
                "/",
                get_with(get_documents, |op| {
                    op.id("get_documents")
                        .description("Placeholder: returns an empty response.")
                })
                .post_with(classify_documents, |op| {
                    op.id("classify_documents")
                        .description("Placeholder: returns an empty response.")
                })
                .options_with(query_documents, |op| {
                    op.id("query_documents")
                        .description("Placeholder: returns an empty response.")
                }),
            )
            .with_state(self)
    }
}

async fn get_documents(State(service): State<DocumentService>) {
    service.get_documents().await;
}

async fn classify_documents(State(service): State<DocumentService>) {
    service.classify_documents().await;
}

async fn query_documents(State(service): State<DocumentService>) {
    service.query_documents().await;
}
