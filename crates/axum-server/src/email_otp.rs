use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};

pub fn routes() -> Router {
    Router::new()
        .route("/auth/email_otp", get(email_otp))
        .route("/auth/email_otp", post(process_email_otp))
}

/***
 * Create a session cookie then send the user an email code 
 * and wait for them to enter it.
 *
 * If the user doesn't exist yet we still need to send an email and set the cookie
 * to deter account enumeration due to timing attacks.
 *
 * So basically login and registration are the same process just the emails
 * will be different.
 */
pub async fn email_otp() -> Result<Html<String>, CustomError> {

    Ok(Html(ui_components::email_otp::email_otp()))
}

/***
 * If the user is able to enter the correct code we know they own that email address
 * 
 * We can create an entry in the user table if one doesn't existsor assign
 * the entry in the session table to an existing users.
 * 
 * We don't have to worry about account enumeration due to timing attacks as we
 * already have confirmation the user owns the address.
 * 
 * We do have to worry about brute forcing, so we have a delay between each check of
 * the code which doubles every attempt.
 */
pub async fn process_email_otp(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
) -> Result<impl IntoResponse, CustomError> {
    let _client = pool.get().await?;

    Ok(Redirect::to(&config.redirect_url))
}
