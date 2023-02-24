use crate::{config, errors::CustomError};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Router,
};
use hyper::{header, StatusCode};
use rand::Rng;
use serde::Deserialize;
use sha2::{Digest, Sha256};

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
    let (otp_code, otp_code_encrypted) = generate_otp(config.secret_key).await?;

    let (session_verifier, hashed_session_verifier) = generate_session_verfier().await;

    let client = pool.get().await?;
    let session_id = db::queries::sessions::create_session()
        .bind(
            &client,
            &hashed_session_verifier.as_ref(),
            &otp_code_encrypted.as_ref(),
            &login_form.email.as_ref(),
        )
        .one()
        .await?;

    // The cookie has our session_id for fast db lookup, then our
    // sessions verifier so an attacker can't easily forge the session.
    let cookie = format!(
        "session={}:{}; SameSite=Strict; Path=/",
        session_id, session_verifier
    );

    let headers = [
        (header::LOCATION, ui_components::EMAIL_OTP),
        (header::SET_COOKIE, &cookie),
    ];

    // Send an email

    // Redirect to the screen ready to receive the users OTP code.
    Ok((StatusCode::SEE_OTHER, headers).into_response())
}

async fn generate_otp(secret: Vec<u8>) -> Result<(u32, String), CustomError> {
    // We generate and OTP code and encrypt it.
    // Encryption helps secure against an attacker who has read only access to the database
    let mut rng = rand::thread_rng();
    let otp_code: u32 = rng.gen_range(10000..99999);
    let otp_encrypted =
        crate::encryption::encrypt(&format!("{}", otp_code), &format!("{}", otp_code), &secret)?;

    Ok((otp_code, otp_encrypted))
}

async fn generate_session_verfier() -> (String, String) {
    // Create a random session verifier
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    let mut hasher = Sha256::new();
    // Hash it to avoid exposing it in the database. i.e. a passive attacker
    // with access to the database won't be able to re-create the session cookie.
    hasher.update(random_bytes);
    let hashed_session_verifier = hex::encode(hasher.finalize());

    (hex::encode(random_bytes), hashed_session_verifier)
}
