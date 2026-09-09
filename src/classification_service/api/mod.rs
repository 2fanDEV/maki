use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    Extension, Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};

use super::ClassificationService;
use crate::{
    api::response::{ApiError, ErrorResponse},
    label_service::LabelService,
    service::Service,
};
use request::{CreateTrainerRequest, ModelPath, TrainerPath};
use response::{ModelMetadata, TrainerMetadata};

pub mod request;
pub mod response;

impl Service for ClassificationService {
    fn api_router(self) -> ApiRouter {
        ApiRouter::new()
            .api_route("/trainers", post_with(create_trainer, |op| op.id("create_trainer")
                .description("Validate label IDs and store an untrained trainer in memory. Does not fit a model.")
                .response::<201, Json<TrainerMetadata>>()
                .response::<400, Json<ErrorResponse>>()
                .response::<422, Json<ErrorResponse>>()
                .response::<500, Json<ErrorResponse>>()))
            .api_route("/trainers/{trainer_id}", get_with(trainer_metadata, |op| op.id("trainer_metadata")
                .description("Read trainer configuration, lifecycle state, error, or resulting model ID.")
                .response::<400, String>()
                .response::<404, Json<ErrorResponse>>()))
            .api_route("/trainers/{trainer_id}/train", post_with(train, |op| op.id("train_model")
                .description("Train and evaluate once, awaiting completion. Returns a new model ID. Disconnecting does not cancel training.")
                .response::<201, Json<ModelMetadata>>()
                .response::<400, String>()
                .response::<404, Json<ErrorResponse>>()
                .response::<409, Json<ErrorResponse>>()
                .response::<422, Json<ErrorResponse>>()
                .response::<500, Json<ErrorResponse>>()))
            .api_route("/models/{model_id}", get_with(model_metadata, |op| op.id("model_metadata")
                .description("Read completed model metadata and evaluation. Resolve current label names through /labels.")
                .response::<400, String>()
                .response::<404, Json<ErrorResponse>>()))
            .with_state(self)
    }
}

async fn create_trainer(
    State(service): State<ClassificationService>,
    Extension(labels): Extension<LabelService>,
    input: Result<Json<CreateTrainerRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<TrainerMetadata>), ApiError> {
    let Json(request) = input?;
    Ok((
        StatusCode::CREATED,
        Json(service.create_trainer(request, &labels).await?),
    ))
}

async fn trainer_metadata(
    State(service): State<ClassificationService>,
    Path(path): Path<TrainerPath>,
) -> Result<Json<TrainerMetadata>, ApiError> {
    Ok(Json(service.trainer_metadata(path.trainer_id).await?))
}

async fn train(
    State(service): State<ClassificationService>,
    Path(path): Path<TrainerPath>,
) -> Result<(StatusCode, Json<ModelMetadata>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(service.train(path.trainer_id).await?),
    ))
}

async fn model_metadata(
    State(service): State<ClassificationService>,
    Path(path): Path<ModelPath>,
) -> Result<Json<ModelMetadata>, ApiError> {
    Ok(Json(service.model_metadata(path.model_id).await?))
}
