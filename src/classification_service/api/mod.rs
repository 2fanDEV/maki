use std::convert::Infallible;

use aide::axum::{
    ApiRouter,
    routing::{get_with, post_with},
};
use axum::{
    Json,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
};
use tokio_stream::{StreamExt, wrappers::WatchStream};

use super::ClassificationService;
use crate::{methods::FittingState, service::Service};
use request::{CreateModelRequest, ModelPath};
use response::{AcceptedModel, ApiError, ErrorResponse, ModelMetadata};

pub mod request;
pub mod response;

impl ClassificationService {
    pub const BASE_PATH: &'static str = "/models";
}

impl Service for ClassificationService {
    fn api_router(self) -> ApiRouter {
        ApiRouter::new()
            .api_route(Self::BASE_PATH, post_with(create_model, |op| {
                op.id("create_model")
                    .description("Start background training. Poll status_url or subscribe to events_url; models are held in memory only.")
                    .response_with::<202, Json<AcceptedModel>, _>(|r| r.description("Model creation accepted; training may still be running."))
                    .response_with::<400, Json<ErrorResponse>, _>(|r| r.description("Invalid JSON, request shape, or unsupported classifier."))
                    .response_with::<422, Json<ErrorResponse>, _>(|r| r.description("Invalid training configuration or dataset."))
            }))
            .api_route("/models/{model_id}", get_with(model_metadata, |op| {
                op.id("model_metadata")
                    .description("Read current training progress, failure, or the completed model's evaluation report.")
                    .response_with::<400, String, _>(|r| r.description("Invalid model UUID."))
                    .response_with::<404, Json<ErrorResponse>, _>(|r| r.description("Unknown model ID."))
            }))
            .api_route("/models/{model_id}/events", get_with(model_events, |op| {
                op.id("model_events")
                    .description("Server-sent events named progress, completed, or failed. Each data field contains JSON ModelMetadata. Sends the latest state on connection, then updates; intermediate updates may be coalesced. Closes after completed or failed. Reconnection receives the latest state without event replay. Disconnecting does not cancel training.")
                    .response_with::<200, Json<ModelMetadata>, _>(|mut r| {
                        let mut media = r.inner().content.shift_remove("application/json").unwrap();
                        // The wire body is SSE text; each event's data is a ModelMetadata object.
                        media.extensions.insert("x-sse-data-schema".into(), serde_json::to_value(media.schema.take().unwrap()).unwrap());
                        media.schema = Some(aide::openapi::SchemaObject {
                            json_schema: schemars::json_schema!({"type": "string"}),
                            example: None,
                            external_docs: None,
                        });
                        r.inner().content.insert("text/event-stream".into(), media);
                        r.description("Stream of model progress and terminal events. Keep-alive comments may be sent.")
                    })
                    .response_with::<400, String, _>(|r| r.description("Invalid model UUID."))
                    .response_with::<404, Json<ErrorResponse>, _>(|r| r.description("Unknown model ID."))
            }))
            .with_state(self)
    }
}

async fn create_model(
    State(service): State<ClassificationService>,
    input: Result<Json<CreateModelRequest>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(request) =
        input.map_err(|error| ApiError(StatusCode::BAD_REQUEST, error.body_text()))?;
    let (model_id, seed) = service
        .create_model(request)
        .await
        .map_err(|error| ApiError(StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;
    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedModel {
            model_id,
            seed,
            status_url: format!("/models/{model_id}"),
            events_url: format!("/models/{model_id}/events"),
        }),
    )
        .into_response())
}

async fn model_metadata(
    State(service): State<ClassificationService>,
    Path(ModelPath { model_id }): Path<ModelPath>,
) -> Result<Json<ModelMetadata>, ApiError> {
    let receiver = service
        .receiver(model_id)
        .await
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "unknown model".into()))?;
    let metadata = receiver.borrow().clone();
    Ok(Json(metadata))
}

async fn model_events(
    State(service): State<ClassificationService>,
    Path(ModelPath { model_id }): Path<ModelPath>,
) -> Result<Response, ApiError> {
    let stream = WatchStream::new(
        service
            .receiver(model_id)
            .await
            .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "unknown model".into()))?,
    )
    .map(|metadata| {
        let event = match metadata.progress.state {
            FittingState::Completed => "completed",
            FittingState::Failed => "failed",
            _ => "progress",
        };
        Ok::<_, Infallible>(
            Event::default()
                .event(event)
                .json_data(metadata)
                .expect("model progress and metrics are finite"),
        )
    });
    Ok(Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response())
}
