use axum::{Router, extract::State, routing::get};

use crate::service::Service;

#[derive(Clone, Default)]
pub struct DocumentService {}

impl Service for DocumentService {
    fn router(self) -> Router {
        Router::new()
            .route(
                "/",
                get(Self::get_documents)
                    .post(Self::classify_documents)
                    .options(Self::query_documents),
            )
            .with_state(self)
    }
}

impl DocumentService {
    pub const BASE_PATH: &str = "/documents";
    async fn classify_documents(State(_service): State<Self>) {}
    async fn get_documents(State(_service): State<Self>) {}
    async fn query_documents(State(_service): State<Self>) {}
}
