use aide::{
    axum::ApiRouter,
    openapi::{Info, OpenApi},
    swagger::Swagger,
};
use axum::{Router, http::header, routing::get};

use crate::{
    classification_service::ClassificationService, document_service::DocumentService,
    service::Service,
};

pub fn router() -> Router {
    let mut api = OpenApi {
        info: Info {
            title: "Maki API".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let router = ApiRouter::new()
        .route("/swagger", Swagger::new("/openapi.json").axum_route())
        .nest(
            DocumentService::BASE_PATH,
            DocumentService::default().api_router(),
        )
        .merge(ClassificationService::default().api_router())
        .finish_api(&mut api);
    let spec = axum::body::Bytes::from(serde_json::to_vec(&api).expect("OpenAPI is serializable"));
    router.route(
        "/openapi.json",
        get(move || {
            let spec = spec.clone();
            async move { ([(header::CONTENT_TYPE, "application/json")], spec) }
        }),
    )
}
