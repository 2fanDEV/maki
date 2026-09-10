use utoipa_axum::{router::OpenApiRouter, routes};

use super::ClassificationService;
use crate::service::Service;

pub mod request;
pub mod response;
mod router;

impl Service for ClassificationService {
    fn api_router(self) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(router::create_trainer))
            .routes(routes!(router::trainer_metadata))
            .routes(routes!(router::train_model))
            .routes(routes!(router::model_metadata))
            .with_state(self)
    }
}
