use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use std::default::Default;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Default)]
pub struct Otp {
    pub code: i32,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Session {
    otp_code: i32,
    otp_code_attempts: i32,
}

pub async fn email_otp() -> Result<HttpResponse> {
    let body = OtpPage {
        form: &Otp::default(),
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_otp(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    session: Option<crate::Session>,
    form: web::Form<Otp>,
) -> Result<HttpResponse, CustomError> {
    if let Some(session) = session {
        if let Ok(uuid) = Uuid::parse_str(&session.session_uuid) {
            if super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await {
                let db_session: Session = sqlx::query_as::<_, Session>(
                    "
                    SELECT otp_code, otp_code_attempts FROM sessions WHERE session_uuid = $1
                    ",
                )
                .bind(uuid)
                .fetch_one(pool.get_ref()) // -> Vec<Person>
                .await?;

                if db_session.otp_code == form.code {
                    sqlx::query(&format!(
                        "
                        UPDATE sessions SET otp_code_confirmed = true WHERE session_uuid = $1
                        "
                    ))
                    .bind(uuid)
                    .execute(pool.get_ref())
                    .await?;

                    return Ok(HttpResponse::SeeOther()
                        .append_header((http::header::LOCATION, config.redirect_url.clone()))
                        .finish());
                } else {
                    sqlx::query(&format!(
                        "
                        UPDATE sessions SET otp_code_attempts = otp_code_attempts + 1 WHERE session_uuid = $1
                        "
                    ))
                    .bind(uuid )
                    .execute(pool.get_ref())
                    .await?;

                    return Ok(HttpResponse::SeeOther()
                        .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                        .finish());
                }
            }
        }
    }

    return Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish());
}

markup::define! {
    OtpPage<'a>(form: &'a  Otp,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Password Reset Request" }

            @forms::NumberInput{ title: "Code", name: "code", value: &form.code.to_string(), help_text: "", errors }

            button.a_button.success[type = "submit"] { "Submit Code" }
        }
    }
}
