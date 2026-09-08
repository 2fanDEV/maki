use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use uuid::Uuid;

use super::request::{ClassifierType, SplitSettings, TfIdfSettings};
use crate::methods::{EvaluationReport, TrainingProgress};

#[derive(schemars::JsonSchema, Clone, Debug, Serialize)]
pub struct ModelMetadata {
    pub model_id: Uuid,
    pub classifier: ClassifierType,
    pub seed: u64,
    pub feature_count: Option<usize>,
    pub split: SplitSettings,
    pub tf_idf: TfIdfSettings,
    pub progress: TrainingProgress,
    pub evaluation: Option<EvaluationReport>,
    pub error: Option<String>,
}

#[derive(schemars::JsonSchema, Serialize)]
pub(super) struct AcceptedModel {
    pub(super) model_id: Uuid,
    pub(super) seed: u64,
    pub(super) status_url: String,
    pub(super) events_url: String,
}

#[derive(Serialize, schemars::JsonSchema)]
pub(super) struct ErrorResponse {
    error: String,
}

pub(super) struct ApiError(pub(super) StatusCode, pub(super) String);
// Status codes are documented on each route because they depend on the operation.
impl aide::OperationOutput for ApiError {
    type Inner = ErrorResponse;
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorResponse { error: self.1 })).into_response()
    }
}
