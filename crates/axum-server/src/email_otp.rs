use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

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

#[derive(Deserialize, Default, Debug)]
pub struct OtpForm {
    pub otp_code: u32,
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
 *
 * We silently fail the otp code check if the user has tried more than 5 times. This
 * is not great for usability but is necessary to stop bot attacks.
 */
pub async fn process_email_otp(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar,
    Form(otp_form): Form<OtpForm>,
) -> Result<impl IntoResponse, CustomError> {
    let client = pool.get().await?;

    let session_from_db =
        crate::session::get_session(&client, &jar, config.secret_key.clone()).await;

    if let Some(session) = session_from_db {
        let code_decrypted = crate::encryption::decrypt(
            &session.otp_code_encrypted,
            "Barricade",
            &config.secret_key,
        )?;

        let otp_code = code_decrypted.parse::<u32>();

        // Do we have a decrypted otp_code?
        if let Ok(otp_code) = otp_code {
            let verified = otp_code == otp_form.otp_code && session.otp_code_attempts < 6;

            // Get the user with that email if they are already registered
            let user = db::queries::users::find_by_email()
                .bind(&client, &session.email.as_ref())
                .opt()
                .await?;

            // If the user existed then it's a login otherwise we register them.
            if let Some(user) = user {
                db::queries::sessions::set_verified_and_increase_attempts()
                    .bind(&client, &verified, &user.id)
                    .await?;
            } else {
                if verified {
                    let user_id = db::queries::users::create_user()
                        .bind(&client, &session.email.as_ref())
                        .one()
                        .await?;
                    db::queries::sessions::set_verified_and_increase_attempts()
                        .bind(&client, &verified, &user_id)
                        .await?;
                }
            }

            if ! verified {
                return Ok(Redirect::to(ui_components::EMAIL_OTP));
            } 
        }
    }

    // If we are enabling end to end encryption then the next step is to
    // get a password from the user
    if config.auth_type == crate::config::AuthType::Encrypted {
        Ok(Redirect::to(ui_components::ENCRYPTION_PASSWORD))
    } else {
        Ok(Redirect::to(&config.redirect_url))
    }
}
