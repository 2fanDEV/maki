use utoipa_axum::{router::OpenApiRouter, routes};

use super::LabelService;
use crate::service::Service;

pub mod request;
pub mod response;
mod router;

impl Service for LabelService {
    fn api_router(self) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(router::create_label, router::list_labels))
            .routes(routes!(
                router::get_label,
                router::rename_label,
                router::delete_label,
            ))
            .with_state(self)
    }
}
