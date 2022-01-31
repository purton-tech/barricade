use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
pub struct User {
    email: String,
    protected_symmetric_key: String,
    protected_ecdsa_private_key: String,
    ecdsa_public_key: String,
    protected_ecdh_private_key: String,
    ecdh_public_key: String,
}

pub async fn decrypt(
    config: web::Data<config::Config>,
    session: Option<crate::Session>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    // If we have a session cookie, try and convert it to a user.
    if let Some(session) = session {
        let logged_user = crate::get_user_by_session(&session, db_pool.get_ref()).await;

        if let Some(logged_user) = logged_user {
            // Make sure they did email otp first.
            if config.email_otp_enabled && !logged_user.otp_code_confirmed {
                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                    .finish());
            }

            if let Some(master_password_hash) = session.master_key_hash {
                let users = sqlx::query_as::<_, User>(&format!(
                    "
                    SELECT 
                        email, 
                        protected_symmetric_key, 
                        protected_ecdsa_private_key, 
                        ecdsa_public_key, 
                        protected_ecdh_private_key, 
                        ecdh_public_key
                    FROM {} WHERE id = $1
                    ",
                    config.user_table_name
                ))
                .bind(logged_user.user_id)
                .fetch_all(db_pool.get_ref()) // -> Vec<Person>
                .await?;

                let unwrapped_protected_symmetric_key = &crate::encryption::kdf_and_unwrap(
                    &users[0].protected_symmetric_key,
                    &master_password_hash,
                    &users[0].ecdh_public_key,
                )
                .await?;

                let unwrapped_protected_ecdh_private_key = &crate::encryption::kdf_and_unwrap(
                    &users[0].protected_ecdh_private_key,
                    &master_password_hash,
                    &users[0].ecdh_public_key,
                )
                .await?;

                let unwrapped_protected_ecdsa_private_key = &crate::encryption::kdf_and_unwrap(
                    &users[0].protected_ecdsa_private_key,
                    &master_password_hash,
                    &users[0].ecdh_public_key,
                )
                .await?;

                let page = DecryptMasterKeyPage {
                    user: &users[0],
                    unwrapped_protected_symmetric_key,
                    unwrapped_protected_ecdh_private_key,
                    unwrapped_protected_ecdsa_private_key,
                };

                return Ok(layouts::session_layout(
                    "Master Key",
                    &page.to_string(),
                    false,
                ));
            }
        }
    }

    // We didn't get the session for some reason.
    Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish())
}

pub async fn process_decryption(config: web::Data<config::Config>) -> Result<HttpResponse> {
    Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, config.redirect_url.clone()))
        .finish())
}

markup::define! {
    DecryptMasterKeyPage<'a>(
        user: &'a User,
        unwrapped_protected_symmetric_key: &'a str,
        unwrapped_protected_ecdh_private_key: &'a str,
        unwrapped_protected_ecdsa_private_key: &'a str) {
        .m_decryption["data-controller" = "master"] {
            h1 { "Decrypting Your Master Key" }
            svg.progress[viewBox="0 0 200 200"] {
                path[class="bg", stroke="#ccc", d="M41 149.5a77 77 0 1 1 117.93 0",  fill="none"] {}
                path["data-target" = "master.path", class="meter", stroke="#09c", d="M41 149.5a77 77 0 1 1 117.93 0",
                    fill="none", "stroke-dasharray"="350", "stroke-dashoffset"="350"] {}
            }
            form[method="post", "data-target" = "master.form"] {
                input["data-target" = "master.protectedSymmetricKey", type="hidden",
                    value=unwrapped_protected_symmetric_key] {}
                input["data-target" = "master.publicECDHKey", type="hidden",
                    value=user.ecdh_public_key.clone()] {}
                input["data-target" = "master.protectedECDHPrivateKey", type="hidden",
                    value=unwrapped_protected_ecdh_private_key] {}
                input["data-target" = "master.publicECDSAKey", type="hidden",
                    value=user.ecdsa_public_key.clone()] {}
                input["data-target" = "master.protectedECDSAPrivateKey", type="hidden",
                    value=unwrapped_protected_ecdsa_private_key] {}
                input["data-target" = "master.email", type="hidden", value=user.email.clone()] {}
            }
        }
    }
}
