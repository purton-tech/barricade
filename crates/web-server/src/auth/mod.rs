pub mod sign_in;
use axum::Router;
use axum_extra::routing::RouterExt;

pub fn routes() -> Router {
    Router::new().typed_get(sign_in::sign_in)
}
