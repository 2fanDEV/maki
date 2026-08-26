use axum::Router;

pub trait Service {
    fn router(self) -> Router;
}
