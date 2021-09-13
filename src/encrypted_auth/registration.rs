use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::default::Default;
use validator::Validate;

#[derive(Serialize, Validate, Deserialize, Default)]
pub struct Registration {
    #[validate(email(message = "Email is not valid"))]
    pub email: String,
    #[validate(length(min = 1, message = "The master_password_hash is invalid"))]
    pub master_password_hash: String,
    #[validate(length(min = 1, message = "The protected_symmetric_key invalid"))]
    pub protected_symmetric_key: String,

    // ECDSA
    #[validate(length(min = 1, message = "The protected_ecdsa_private_key is invalid"))]
    pub protected_ecdsa_private_key: String,
    #[validate(length(min = 1, message = "The ecdsa_public_key is invalid"))]
    pub ecdsa_public_key: String,

    // ECDH
    #[validate(length(min = 1, message = "The protected_ecdh_private_key is invalid"))]
    pub protected_ecdh_private_key: String,
    #[validate(length(min = 1, message = "The ecdh_public_key is invalid"))]
    pub ecdh_public_key: String,
}

pub async fn registration() -> Result<HttpResponse> {
    let body = RegistrationPage {
        form: &Registration::default(),
    };

    Ok(layouts::session_layout("Registration", &body.to_string()))
}
#[derive(sqlx::FromRow)]
struct InsertedUser {
    id: i32,
}

pub async fn process_registration(
    config: web::Data<config::Config>,
    db_pool: web::Data<PgPool>,
    user: web::Form<Registration>,
    identity: Identity,
) -> Result<HttpResponse, CustomError> {
    let registration = Registration {
        email: user.email.clone(),
        ..Default::default()
    };

    match user.validate() {
        Ok(_) => {
            // Hash with Argon2id otherwise someone with readonly access to the db
            // could use this directly as a logon token.
            let master_password_hash =
                crate::encryption::password_hash(&user.master_password_hash, false).await?;

            // Encrypt private keys one more time. Extra protection against brute
            // force of the master password.
            let server_wrapped_protected_symmetric_key = crate::encryption::kdf_and_wrap(
                &user.protected_symmetric_key,
                &user.master_password_hash,
                &user.ecdh_public_key,
            )
            .await?;

            let server_wrapped_protected_ecdh_private_key = crate::encryption::kdf_and_wrap(
                &user.protected_ecdh_private_key,
                &user.master_password_hash,
                &user.ecdh_public_key,
            )
            .await?;

            let server_wrapped_protected_ecdsa_private_key = crate::encryption::kdf_and_wrap(
                &user.protected_ecdsa_private_key,
                &user.master_password_hash,
                &user.ecdh_public_key,
            )
            .await?;

            let db_user = sqlx::query_as::<_, InsertedUser>(&format!(
                "
                INSERT INTO {} 
                    (email, 
                    master_password_hash, 
                    protected_symmetric_key, 
                    protected_ecdh_private_key, 
                    ecdh_public_key,
                    protected_ecdsa_private_key, 
                    ecdsa_public_key)
                VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING id
                ",
                config.user_table_name
            ))
            .bind(&user.email)
            .bind(&master_password_hash)
            .bind(server_wrapped_protected_symmetric_key)
            .bind(server_wrapped_protected_ecdh_private_key)
            .bind(&user.ecdh_public_key)
            .bind(server_wrapped_protected_ecdsa_private_key)
            .bind(&user.ecdsa_public_key)
            .fetch_one(db_pool.get_ref())
            .await?;

            crate::auth::login::create_session(
                &config,
                db_pool,
                identity,
                db_user.id,
                Some(user.master_password_hash.clone()),
            )
            .await?;

            if config.email_otp_enabled {
                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                    .finish());
            }

            Ok(HttpResponse::SeeOther()
                .append_header((http::header::LOCATION, crate::DECRYPT_MASTER_KEY_URL))
                .finish())
        }
        Err(_) => {
            let body = RegistrationPage {
                form: &registration,
            };

            Ok(layouts::session_layout("Registration", &body.to_string()))
        }
    }
}

markup::define! {
    RegistrationPage<'a>(form: &'a  Registration) {
        div["data-controller" = "registration password"] {
            form.m_authentication {

                h1 { "Register" }

                label[for="email"] { "Email" }
                input#email[name = "email", value = &form.email, "data-target" = "registration.email"] {}
                span.a_help_text { "You'll use your email address to log in." }

                label[for="password"] { "Master Password" }
                input#password[name="password", type="password",
                    "data-action"="keyup->password#keyPress",
                    "data-target" = "registration.password password.password"] {}
                span.a_help_text["data-target" = "password.help"] { "The master password is the password we use to generate your private keys. It is very important that you do not forget your master password. There is no way to recover the password in the event that you forget it." }
                span.a_help_text["data-target" = "password.warning"] {}
                span.a_help_text["data-target" = "password.suggestions"] {}

                label[for="confirm_password"] { "Re-type Master Password" }
                input#confirm_password[name="confirm_password", type="password", "data-target" = "registration.confirmPassword"] {}

                button.a_button.success[type = "submit", "data-target" = "registration.button password.button",
                    "data-action" = "registration#register"] { "Sign Up" }

                a[href=crate::SIGN_IN_URL] { "Sign In Instead" }
            }
            form[method = "post", "data-target" = "registration.form"] {
                input[name="master_password_hash", "data-target" = "registration.masterPasswordHash", type="hidden"] {}
                input[name="email", "data-target" = "registration.emailCopy", type="hidden"] {}
                input[name="protected_symmetric_key", "data-target" = "registration.protectedSymmetricKey", type="hidden"] {}

                // ECDH
                input[name="protected_ecdh_private_key",
                    "data-target" = "registration.protectedECDHPrivateKey",
                    type="hidden"] {}
                input[name="ecdh_public_key",
                    "data-target" = "registration.publicECDHKey",
                    type="hidden"] {}

                // ECDSA
                input[name="protected_ecdsa_private_key",
                    "data-target" = "registration.protectedECDSAPrivateKey",
                    type="hidden"] {}
                input[name="ecdsa_public_key",
                    "data-target" = "registration.publicECDSAKey",
                    type="hidden"] {}
            }
        }
        script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
    }
}
