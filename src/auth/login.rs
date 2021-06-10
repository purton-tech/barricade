use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Result};
use bcrypt::verify;
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(sqlx::FromRow)]
pub struct User {
    id: i32,
    hashed_password: String,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(sqlx::FromRow)]
struct InsertedSession {
    session_uuid: Uuid,
}

pub async fn login(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = LoginPage {
        form: &Login::default(),
        hcaptcha_config: &config.hcaptcha_config,
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn create_session(
    pool: web::Data<PgPool>,
    identity: Identity,
    user_id: i32,
) -> Result<(), CustomError> {
    let session = sqlx::query_as::<_, InsertedSession>(
        "
            INSERT INTO sessions (user_id)
            VALUES($1) RETURNING session_uuid
        ",
    )
    .bind(user_id)
    .fetch_one(pool.get_ref())
    .await?;

    identity.remember(session.session_uuid.to_string());

    Ok(())
}

pub async fn process_login(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    identity: Identity,
    form: web::Form<Login>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    if super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await {
        let users = sqlx::query_as::<_, User>(&format!(
            "
            SELECT id, hashed_password FROM {} WHERE email = $1
            ",
            config.user_table_name
        ))
        .bind(&form.email.to_lowercase())
        .fetch_all(pool.get_ref()) // -> Vec<Person>
        .await?;

        if !users.is_empty() {
            let valid = verify(&form.password, &users[0].hashed_password)
                .map_err(|_| CustomError::Unauthorized)?;

            if valid {
                // Generate a session

                create_session(pool, identity, users[0].id).await?;

                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, config.redirect_url.clone()))
                    .finish());
            }
        }

        validation_errors.add(
            "email",
            ValidationError {
                message: Some(Cow::from("Invalid email or password")),
                code: Cow::from("0"),
                params: Default::default(),
            },
        );
    } else {
        validation_errors.add(
            "email",
            ValidationError {
                message: Some(Cow::from("Invalid hCaptcha")),
                code: Cow::from("0"),
                params: Default::default(),
            },
        );
    }

    let login = Login {
        email: form.email.clone(),
        ..Default::default()
    };

    let body = LoginPage {
        form: &login,
        hcaptcha_config: &config.hcaptcha_config,
        errors: &validation_errors,
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

markup::define! {
    LoginPage<'a>(form: &'a  Login,
    hcaptcha_config: &'a Option<config::HCaptchaConfig>,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Sign In" }

            @forms::EmailInput{ title: "Email", name: "email", value: &form.email, help_text: "", errors }
            @forms::PasswordInput{ title: "Password", name: "password", value: &form.password, help_text: "", errors }

            @if let Some(hcaptcha) = hcaptcha_config {
                div."h-captcha"["data-sitekey"=&hcaptcha.hcaptcha_site_key] {}
            }
            button.a_button.success[type = "submit"] { "Log In" }
            div {
                a[href=crate::SIGN_UP_URL] { "Sign Up" }
                { " | " }
                a[href=crate::RESET_REQUEST_URL] { "Reset Password" }
            }
        }

        @if let Some(_) = hcaptcha_config {
            script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
        }
    }
}
