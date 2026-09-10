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
        (status = 201, body = LabelResponse),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn create_label(
    State(service): State<LabelService>,
    input: Result<Json<LabelInput>, JsonRejection>,
) -> Result<(StatusCode, Json<LabelResponse>), ApiError> {
    let Json(input) = input?;
    Ok((
        StatusCode::CREATED,
        Json(service.create(&input.name).await?.into()),
    ))
}

/// List all labels ordered by ID.
#[utoipa::path(
    get,
    path = "/labels",
    responses(
        (status = 200, body = [LabelResponse]),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn list_labels(
    State(service): State<LabelService>,
) -> Result<Json<Vec<LabelResponse>>, ApiError> {
    Ok(Json(
        service.list().await?.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/labels/{id}",
    params(("id" = String, Path)),
    responses(
        (status = 200, body = LabelResponse),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 404, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn get_label(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
) -> Result<Json<LabelResponse>, ApiError> {
    Ok(Json(service.get(object_id(&path.id)?).await?.into()))
}

#[utoipa::path(
    patch,
    path = "/labels/{id}",
    params(("id" = String, Path)),
    request_body = LabelInput,
    responses(
        (status = 200, body = LabelResponse),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 404, body = crate::api::response::ErrorResponse),
        (status = 422, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn rename_label(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
    input: Result<Json<LabelInput>, JsonRejection>,
) -> Result<Json<LabelResponse>, ApiError> {
    let Json(input) = input?;
    Ok(Json(
        service
            .rename(object_id(&path.id)?, &input.name)
            .await?
            .into(),
    ))
}

/// Delete a label. Document/trainer reference checks are not implemented.
#[utoipa::path(
    delete,
    path = "/labels/{id}",
    params(("id" = String, Path)),
    responses(
        (status = 204),
        (status = 400, body = crate::api::response::ErrorResponse),
        (status = 404, body = crate::api::response::ErrorResponse),
        (status = 500, body = crate::api::response::ErrorResponse),
    )
)]
pub(super) async fn delete_label(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
) -> Result<StatusCode, ApiError> {
    service.delete(object_id(&path.id)?).await?;
    Ok(StatusCode::NO_CONTENT)
}
