mod document_service;
mod service;

use axum::Router;
use document_service::DocumentService;

use crate::service::Service;

#[tokio::main]
async fn main() {
    env_logger::init();
    let document_service = DocumentService::default();
    let app = Router::new().nest(DocumentService::BASE_PATH, document_service.router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
