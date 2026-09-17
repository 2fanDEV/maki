use axum::extract::DefaultBodyLimit;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::DocumentService;
use crate::service::Service;

pub mod request;
pub mod response;
mod router;

impl DocumentService {
    pub const BASE_PATH: &str = "/documents";
    const MAX_UPLOAD_SIZE: usize = 25 * 1024 * 1024;
}

impl Service for DocumentService {
    fn api_router(self) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(router::upload_document))
            .layer(DefaultBodyLimit::max(Self::MAX_UPLOAD_SIZE))
            .with_state(self)
    }
}
