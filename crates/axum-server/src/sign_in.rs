use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};

pub fn routes() -> Router {
    Router::new()
        .route(ui_components::SIGN_IN, get(sign_in))
        .route(ui_components::SIGN_IN, post(process_sign_in))
}

/***
 * If the user already has a cookie and that cookie is valid then redirect them to
 * the DESTINATION_URL
 *
 * If not show a email form and start the login process
 * 
 * Here we need to worry that an attacker will use our form to annoy our users.
 * To defeat this we'll need a Captcha.
 */
pub async fn sign_in(Extension(pool): Extension<db::Pool>) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;

    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Html(ui_components::sign_in::sign_in(users)))
}

pub async fn process_sign_in(
    Extension(pool): Extension<db::Pool>,
) -> Result<impl IntoResponse, CustomError> {
    let _client = pool.get().await?;

    Ok(Redirect::to(ui_components::EMAIL_OTP))
}
