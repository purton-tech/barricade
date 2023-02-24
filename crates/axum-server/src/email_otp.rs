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
        .route("/auth/email_otp", get(email_otp))
        .route("/auth/email_otp", post(process_email_otp))
}

/***
 * At this point we have a session cookie but it's not validated
 *
 * The user has been sent an email that contains their OTP code.
 *
 * We need to securely validate that code whilst being careful of timing
 * attacks and brute force.
 */
pub async fn email_otp() -> Result<Html<String>, CustomError> {
    Ok(Html(ui_components::email_otp::email_otp()))
}

/***
 * If the user is able to enter the correct code we know they own that email address
 * 
 * We can create an entry in the user table if one doesn't exist or assign
 * the entry in the session table to an existing users.
 * 
 * We do have to worry about account enumeration due to timing attacks but not
 * if we check the otp code first.
 * 
 * We have to worry about brute forcing, so we have a delay between each check of
 * the code which doubles every attempt.
 */
pub async fn process_email_otp(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar
) -> Result<impl IntoResponse, CustomError> {
    let client = pool.get().await?;

    // get the session id and the verifier from the cookie.
    if let Some((id, verifier)) = get_session(&jar).await {
        let session_from_db = db::queries::sessions::get_session()
            .bind(
                &client,
                &(id as i32)
            )
            .one()
            .await?;

        dbg!(session_from_db);
    }

    Ok(Redirect::to(&config.redirect_url))
}

pub async fn get_session(jar: &CookieJar) -> Option<(u64, String)> {

    if let Some(session) = jar.get("session") {
        let mut split = session.value().split(":");
        if let (Some(id), Some(verifier)) = (split.next(), split.next()) {

            if let Ok(id) = id.parse::<u64>() {
                return Some((id, verifier.to_string()));
            }
        }
    }
    None
}
