use axum::Router;
use maki::{
    classification_service::ClassificationService, document_service::DocumentService,
    service::Service,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    let document_service = DocumentService::default();
    let app = Router::new()
        .nest(DocumentService::BASE_PATH, document_service.router())
        .merge(ClassificationService::default().router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
