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
    #[validate(length(min = 1, message = "The private key is invalid"))]
    pub encrypted_private_key: String,
    #[validate(length(min = 1, message = "The public key is invalid"))]
    pub public_key: String,
    #[validate(length(min = 1, message = "The init vector is invalid"))]
    pub init_vector: String,
    #[validate(length(min = 1, message = "The blind index is invalid"))]
    pub blind_index: String,
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
            let user = sqlx::query_as::<_, InsertedUser>(
                &format!(
                    "
                    INSERT INTO {} (email, encrypted_private_key, public_key, blind_index, init_vector)
                    VALUES($1, $2, $3, $4, $5) RETURNING id
                    ",
                    config.user_table_name
                )
            )
            .bind(&user.email)
            .bind(&user.encrypted_private_key)
            .bind(&user.public_key)
            .bind(&user.blind_index)
            .bind(&user.init_vector)
            .fetch_one(db_pool.get_ref())
            .await?;

            crate::auth::login::create_session(db_pool, identity, user.id).await?;

            Ok(HttpResponse::SeeOther()
                .append_header((http::header::LOCATION, config.redirect_url.clone()))
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
        div["data-controller" = "registration"] {
            form.m_authentication {

                h1 { "Register" }

                label[for="email"] { "Email" }
                input[name = "email", value = &form.email, "data-target" = "registration.email"] {}

                label[for="password"] { "Password" }
                input[name="password", type="password", "data-target" = "registration.password"] {}

                label[for="confirm_password"] { "Confirm Password" }
                input[name="confirm_password", type="password", "data-target" = "registration.confirmPassword"] {}

                button.a_button.success[type = "submit", "data-target" = "registration.button",
                    "data-action" = "registration#register"] { "Sign Up" }

                a[href=crate::SIGN_IN_URL] { "Sign In Instead" }
            }
            form[method = "post", "data-target" = "registration.form"] {
                input[name="encrypted_private_key", "data-target" = "registration.encryptedPrivateKey", type="hidden"] {}
                input[name="public_key", "data-target" = "registration.publicKey", type="hidden"] {}
                input[name="init_vector", "data-target" = "registration.initVector", type="hidden"] {}
                input[name="email", "data-target" = "registration.emailCopy", type="hidden"] {}
                input[name="blind_index", "data-target" = "registration.blindIndex", type="hidden"] {}
            }
        }
        script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
    }
}
