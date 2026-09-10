use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::service::ServiceError;

#[derive(Serialize, utoipa::ToSchema)]
pub(crate) struct ErrorResponse {
    error: String,
}

pub(crate) struct ApiError(pub StatusCode, pub String);

impl From<JsonRejection> for ApiError {
    fn from(error: JsonRejection) -> Self {
        Self(StatusCode::BAD_REQUEST, error.body_text())
    }
}

impl From<ServiceError> for ApiError {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::InvalidInput(message) => Self(StatusCode::UNPROCESSABLE_ENTITY, message),
            ServiceError::NotFound(message) => Self(StatusCode::NOT_FOUND, message.into()),
            ServiceError::Conflict(message) => Self(StatusCode::CONFLICT, message.into()),
            ServiceError::Internal(error) => {
                log::error!("service operation failed: {error:#}");
                Self(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorResponse { error: self.1 })).into_response()
    }
}
