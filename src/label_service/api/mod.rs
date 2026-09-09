use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
};
use mongodb::bson::oid::ObjectId;

use super::LabelService;
use crate::{
    api::response::{ApiError, ErrorResponse},
    service::Service,
};
use request::{LabelInput, LabelPath};
use response::LabelResponse;

pub mod request;
pub mod response;

impl Service for LabelService {
    fn api_router(self) -> ApiRouter {
        ApiRouter::new()
            .api_route("/labels", post_with(create, |op| op.id("create_label")
                .description("Create a label with a MongoDB-generated ID. Duplicate names are allowed.")
                .response::<201, Json<LabelResponse>>()
                .response::<400, Json<ErrorResponse>>()
                .response::<422, Json<ErrorResponse>>()
                .response::<500, Json<ErrorResponse>>())
                .get_with(list, |op| op.id("list_labels")
                    .description("List all labels ordered by ID.")
                    .response::<500, Json<ErrorResponse>>()))
            .api_route("/labels/{id}", get_with(get, |op| op.id("get_label")
                .response::<400, Json<ErrorResponse>>()
                .response::<404, Json<ErrorResponse>>()
                .response::<500, Json<ErrorResponse>>())
                .patch_with(rename, |op| op.id("rename_label")
                    .response::<400, Json<ErrorResponse>>()
                    .response::<404, Json<ErrorResponse>>()
                    .response::<422, Json<ErrorResponse>>()
                    .response::<500, Json<ErrorResponse>>())
                .delete_with(delete, |op| op.id("delete_label")
                    .description("Delete a label. Document/trainer reference checks are not implemented.")
                    .response::<204, ()>()
                    .response::<400, Json<ErrorResponse>>()
                    .response::<404, Json<ErrorResponse>>()
                    .response::<500, Json<ErrorResponse>>()))
            .with_state(self)
    }
}

fn object_id(id: &str) -> Result<ObjectId, ApiError> {
    ObjectId::parse_str(id)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "invalid label ObjectId".into()))
}

async fn create(
    State(service): State<LabelService>,
    input: Result<Json<LabelInput>, JsonRejection>,
) -> Result<(StatusCode, Json<LabelResponse>), ApiError> {
    let Json(input) = input?;
    Ok((
        StatusCode::CREATED,
        Json(service.create(&input.name).await?.into()),
    ))
}

async fn list(State(service): State<LabelService>) -> Result<Json<Vec<LabelResponse>>, ApiError> {
    Ok(Json(
        service.list().await?.into_iter().map(Into::into).collect(),
    ))
}

async fn get(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
) -> Result<Json<LabelResponse>, ApiError> {
    Ok(Json(service.get(object_id(&path.id)?).await?.into()))
}

async fn rename(
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

async fn delete(
    State(service): State<LabelService>,
    Path(path): Path<LabelPath>,
) -> Result<StatusCode, ApiError> {
    service.delete(object_id(&path.id)?).await?;
    Ok(StatusCode::NO_CONTENT)
}
