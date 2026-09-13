use axum::Router;
use utoipa_axum::router::OpenApiRouter;

#[derive(Debug)]
pub enum ServiceError {
    InvalidInput(String),
    NotFound(&'static str),
    Conflict(&'static str),
    Internal(anyhow::Error),
}

impl From<mongodb::error::Error> for ServiceError {
    fn from(error: mongodb::error::Error) -> Self {
        Self::Internal(error.into())
    }
}

pub trait Service: Sized {
    fn api_router(self) -> OpenApiRouter;

    fn router(self) -> Router {
        self.api_router().into()
    }
}
