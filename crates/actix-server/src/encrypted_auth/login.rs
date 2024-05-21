use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub master_password_hash: String,
}

#[derive(sqlx::FromRow)]
struct LoginUser {
    id: i32,
    master_password_hash: String,
}

pub async fn login(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = LoginPage {
        form: &Login::default(),
        errors: None,
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

pub async fn process_login(
    config: web::Data<config::Config>,
    form: web::Form<Login>,
    identity: Identity,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    let users = sqlx::query_as::<_, LoginUser>(&format!(
        "
        SELECT id, master_password_hash FROM {} WHERE email = $1
        ",
        config.user_table_name
    ))
    .bind(&form.email.to_lowercase())
    .fetch_all(db_pool.get_ref()) // -> Vec<Person>
    .await?;

    if !users.is_empty() {
        // We don't store the master password hash, we store
        // the Argon2id hash of the hash.
        if crate::encryption::verify_hash(
            &form.master_password_hash,
            &users[0].master_password_hash,
            false,
        )
        .await?
        {
            crate::auth::login::create_session(
                &config,
                db_pool,
                identity,
                users[0].id,
                Some(form.master_password_hash.clone()),
            )
            .await?;

            if config.email_otp_enabled {
                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                    .finish());
            }

            return Ok(HttpResponse::SeeOther()
                .append_header((http::header::LOCATION, crate::DECRYPT_MASTER_KEY_URL))
                .finish());
        }
    }

    let mut validation_errors = ValidationErrors::default();

    validation_errors.add(
        "email",
        ValidationError {
            message: Some(Cow::from("Invalid email or password")),
            code: Cow::from("0"),
            params: Default::default(),
        },
    );

    let login = Login {
        email: form.email.clone(),
        ..Default::default()
    };

    let body = LoginPage {
        form: &login,
        errors: Some(validation_errors),
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

markup::define! {
    LoginPage<'a>(form: &'a  Login, errors: Option<ValidationErrors>) {
        div["data-controller" = "login"] {
            form.m_authentication {

                h1 { "Sign In" }

                label[for="email"] { "Email" }
                @if errors.is_none() {
                    input[id="email",
                    name = "email",
                    autocomplete= "username",
                    value = forms::escape(&form.email),
                    "data-target" = "login.email"] {}
                } else {
                    input.error[id="email",
                        name = "email",
                        type="email",
                        autocomplete= "username",
                        value = &form.email,
                        "data-target" = "login.email"] {}
                    span.error { "Invalid email or password" }
                }

                label[for="password"] { "Password" }
                input[id="password",
                    name = "password",
                    autocomplete= "current-password",
                    type="password",
                    "data-target" = "login.password"] {}

                button.a_button.success[type = "submit",
                    "data-target" = "login.button",
                    "data-action" = "login#login"] { "Log In" }
                div {
                    a[href=crate::SIGN_UP_URL] { "Sign Up" }
                    { " | " }
                    a[href=crate::SIGN_UP_URL] { "Reset Password" }
                }
            }
            form[method = "post", "data-target" = "login.form"] {
                input[name="email", "data-target" = "login.emailCopy", type="hidden"] {}
                input[name="master_password_hash", "data-target" = "login.masterPasswordHash", type="hidden"] {}
            }
        }
    }
}
