use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;

use super::{
    request::{LabelInput, LabelPath},
    response::LabelResponse,
};
use crate::{api::response::ApiError, label_service::LabelService};

fn object_id(id: &str) -> Result<ObjectId, ApiError> {
    ObjectId::parse_str(id)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "invalid label ObjectId".into()))
}

/// Create a label with a MongoDB-generated ID. Duplicate names are allowed.
#[utoipa::path(
    post,
    path = "/labels",
    request_body = LabelInput,
    responses(
        (status = 201),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn create_label(
    State(service): State<LabelService>,
    input: Result<Json<LabelInput>, JsonRejection>,
) -> Result<StatusCode, ApiError> {
    let Json(input) = input?;
    service.create(&input.name).await?;
    Ok(StatusCode::CREATED)
}

/// Delete a label. Document/trainer reference checks are not implemented.
#[utoipa::path(
    delete,
    path = "/labels/{id}",
    params(("id" = String, Path)),
    responses(
        (status = 200, body = LabelResponse),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 404, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn delete_label(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
) -> Result<Json<LabelResponse>, ApiError> {
    Ok(Json(service.delete(object_id(&path.id)?).await?.into()))
}
