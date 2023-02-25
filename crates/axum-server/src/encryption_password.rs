use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;

pub fn routes() -> Router {
    Router::new()
        .route("/auth/encryption_password", get(encryption_password))
        .route("/auth/encryption_password", post(process_encryption_password))
}
pub async fn encryption_password() -> Result<Html<String>, CustomError> {
    Ok(Html(ui_components::encryption_password::encryption_password()))
}

pub async fn process_encryption_password(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar
) -> Result<impl IntoResponse, CustomError> {
    let client = pool.get().await?;

    Ok(Redirect::to(&config.redirect_url))
}
