use axum::{Extension, Router, http::Method};
use mongodb::Database;
use tower_http::cors::{self, AllowOrigin, Any, CorsLayer};
use utoipa::openapi::{Info, OpenApi, path::Paths};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    classification_service::ClassificationService, document_service::DocumentService,
    label_service::LabelService, service::Service,
};

pub(crate) mod response;

pub fn router(database: Database, documents: DocumentService) -> Router {
    let labels = LabelService::new(&database);
    let api = OpenApi::new(
        Info::new("Maki API", env!("CARGO_PKG_VERSION")),
        Paths::new(),
    );
    let cors_layer = CorsLayer::new()
        .allow_methods([Method::OPTIONS, Method::GET, Method::POST])
        .allow_origin(Any);

    let (router, api) = OpenApiRouter::with_openapi(api)
        .nest(DocumentService::BASE_PATH, documents.api_router())
        .merge(ClassificationService::default().api_router())
        .merge(labels.clone().api_router())
        .layer(cors_layer)
        .layer(Extension(labels))
        .layer(Extension(database))
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api))
        .layer(CorsLayer::permissive())
}
