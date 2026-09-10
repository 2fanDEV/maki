use utoipa_axum::{router::OpenApiRouter, routes};

use super::DocumentService;
use crate::service::Service;

pub mod request;
pub mod response;
mod router;

impl DocumentService {
    pub const BASE_PATH: &str = "/documents";
}

impl Service for DocumentService {
    fn api_router(self) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(
                router::get_documents,
                router::classify_documents,
                router::query_documents,
            ))
            .with_state(self)
    }
}
