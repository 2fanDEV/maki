use axum::{
    Extension, Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};

use super::{
    request::{CreateTrainerRequest, ModelPath, TrainerPath},
    response::{ModelMetadata, TrainerMetadata},
};
use crate::{
    api::response::ApiError, classification_service::ClassificationService,
    label_service::LabelService,
};

/// Validate label IDs and store an untrained trainer in memory. Does not fit a model.
#[utoipa::path(
    post,
    path = "/trainers",
    request_body = CreateTrainerRequest,
    responses(
        (status = 201, body = TrainerMetadata),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn create_trainer(
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

/// Read trainer configuration, lifecycle state, error, or resulting model ID.
#[utoipa::path(
    get,
    path = "/trainers/{trainer_id}",
    params(("trainer_id" = uuid::Uuid, Path)),
    responses(
        (status = 200, body = TrainerMetadata),
        (status = 400, body = String, content_type = "text/plain"),
        (status = 404, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn trainer_metadata(
    State(service): State<ClassificationService>,
    Path(path): Path<TrainerPath>,
) -> Result<Json<TrainerMetadata>, ApiError> {
    Ok(Json(service.trainer_metadata(path.trainer_id).await?))
}

/// Train and evaluate once, awaiting completion. Returns a new model ID. Disconnecting does not cancel training.
#[utoipa::path(
    post,
    path = "/trainers/{trainer_id}/train",
    params(("trainer_id" = uuid::Uuid, Path)),
    responses(
        (status = 201, body = ModelMetadata),
        (status = 400, body = String, content_type = "text/plain"),
        (status = 404, body = crate::api::response::ErrorResponse),
        (status = 409, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn train_model(
    State(service): State<ClassificationService>,
    Path(path): Path<TrainerPath>,
) -> Result<(StatusCode, Json<ModelMetadata>), ApiError> {
    Ok((
        StatusCode::CREATED,
        Json(service.train(path.trainer_id).await?),
    ))
}

/// Read completed model metadata and evaluation. Resolve current label names through /labels.
#[utoipa::path(
    get,
    path = "/models/{model_id}",
    params(("model_id" = uuid::Uuid, Path)),
    responses(
        (status = 200, body = ModelMetadata),
        (status = 400, body = String, content_type = "text/plain"),
        (status = 404, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn model_metadata(
    State(service): State<ClassificationService>,
    Path(path): Path<ModelPath>,
) -> Result<Json<ModelMetadata>, ApiError> {
    Ok(Json(service.model_metadata(path.model_id).await?))
}
