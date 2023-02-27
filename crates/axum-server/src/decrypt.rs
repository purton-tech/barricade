use crate::{config, errors::CustomError};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use hyper::{header, StatusCode};
use serde::Deserialize;

use lettre::Message;

pub fn routes() -> Router {
    Router::new()
        .route(ui_components::SIGN_IN, get(sign_in))
        .route(ui_components::SIGN_IN, post(process_sign_in))
}

#[derive(Deserialize, Default, Debug)]
pub struct LoginForm {
    pub email: String,
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

    Ok(Html(ui_components::sign_in::sign_in()))
}