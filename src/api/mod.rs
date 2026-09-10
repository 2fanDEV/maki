use axum::{Extension, Router};
use mongodb::Database;
use utoipa::openapi::{Info, OpenApi, path::Paths};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    classification_service::ClassificationService, document_service::DocumentService,
    label_service::LabelService, service::Service,
};

pub(crate) mod response;

pub fn router(database: Database) -> Router {
    let labels = LabelService::new(&database);
    let api = OpenApi::new(
        Info::new("Maki API", env!("CARGO_PKG_VERSION")),
        Paths::new(),
    );
    let (router, api) = OpenApiRouter::with_openapi(api)
        .nest(
            DocumentService::BASE_PATH,
            DocumentService::default().api_router(),
        )
        .merge(ClassificationService::default().api_router())
        .merge(labels.clone().api_router())
        .layer(Extension(labels))
        .layer(Extension(database))
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger").url("/openapi.json", api))
}
