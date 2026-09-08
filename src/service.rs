use aide::axum::ApiRouter;
use axum::Router;

pub trait Service: Sized {
    fn api_router(self) -> ApiRouter;

    fn router(self) -> Router {
        self.api_router().into()
    }
}
