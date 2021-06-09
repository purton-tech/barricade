use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Reset {
    pub email: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    reset_password_token: Option<Uuid>,
}

pub async fn reset_request(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = ResetPage {
        form: &Reset::default(),
        hcaptcha_config: &config.hcaptcha_config,
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_request(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Reset>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    if super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await {
        let users = sqlx::query_as::<_, User>(&format!(
            "
            UPDATE {} SET 
                reset_password_token = gen_random_uuid(),
                reset_password_sent_at = now()
            WHERE 
                email = $1 
            RETURNING reset_password_token
            ",
            config.user_table_name
        ))
        .bind(&form.email.to_lowercase())
        .fetch_all(pool.get_ref()) // -> Vec<Person>
        .await?;

        dbg!(users);

        return Ok(HttpResponse::SeeOther()
            .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
            .finish());
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

    let login = Reset {
        email: form.email.clone(),
        ..Default::default()
    };

    let body = ResetPage {
        form: &login,
        hcaptcha_config: &config.hcaptcha_config,
        errors: &validation_errors,
    };

    if validation_errors.is_empty() {
        return Ok(HttpResponse::SeeOther()
            .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
            .finish());
    }

    Ok(layouts::session_layout("Login", &body.to_string()))
}

markup::define! {
    ResetPage<'a>(form: &'a  Reset,
    hcaptcha_config: &'a Option<config::HCaptchaConfig>,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Password Reset Request" }

            @forms::EmailInput{ title: "Email", name: "email", value: &form.email, help_text: "", errors }

            @if let Some(hcaptcha) = hcaptcha_config {
                div."h-captcha"["data-sitekey"=&hcaptcha.hcaptcha_site_key] {}
            }
            button.a_button.success[type = "submit"] { "Request Password Reset" }
            div {
                a[href=crate::SIGN_UP_URL] { "Sign Up" }
                { " | " }
                a[href=crate::SIGN_IN_URL] { "Sign In" }
            }
        }

        @if let Some(_) = hcaptcha_config {
            script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
        }
    }
}
