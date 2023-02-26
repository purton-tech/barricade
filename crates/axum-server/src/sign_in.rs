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

    let users = db::queries::users::get_users().bind(&client).all().await?;

    Ok(Html(ui_components::sign_in::sign_in(users)))
}

/**
 * We have the users email so create a session and pass thme over
 * to email otp verfication
 */
pub async fn process_sign_in(
    Extension(pool): Extension<db::Pool>,
    Extension(config): Extension<config::Config>,
    Form(login_form): Form<LoginForm>,
) -> Result<impl IntoResponse, CustomError> {
    
    let client = pool.get().await?;
    let (cookie, otp_code) = crate::session::create_session(&client, &login_form.email, config.secret_key).await?;

    if let Some(smtp_config) = &config.smtp_config {
        // Send an email
    
        let email = Message::builder()
            .from(smtp_config.from_email.clone())
            .to(login_form.email.parse().unwrap())
            .subject("Your confirmation code")
            .body(
                format!(
                    "
                Your confirmation code is {}
                ",
                    otp_code
                )
                .trim()
                .to_string(),
            )
            .unwrap();

        let result = crate::email::send_email(smtp_config, email);

        tracing::debug!("Email -> {:?}", result);
    } else {
        tracing::error!("SMTP is not configured");
    }

    // Redirect to the screen ready to receive the users OTP code.

    let headers = [
        (header::LOCATION, ui_components::EMAIL_OTP.to_string()),
        (header::SET_COOKIE, cookie),
    ];

    Ok((StatusCode::SEE_OTHER, headers).into_response())
}
