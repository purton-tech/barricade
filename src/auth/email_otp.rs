use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use lettre::Message;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::default::Default;
use validator::ValidationErrors;

pub static INVALID_USER_ID: i32 = -1000;

#[derive(Serialize, Deserialize, Default)]
pub struct Otp {
    pub code: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Session {
    user_id: i32,
    otp_code_encrypted: String,
    otp_code_attempts: i32,
    otp_code_sent: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    email: String,
}

pub async fn email_otp(
    config: web::Data<config::Config>,
    session: Option<crate::Session>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    if let Some(session) = session {
        if let Some(user_session) = crate::get_user_by_session(&session, pool.get_ref()).await {
            if !user_session.otp_code_sent {
                sqlx::query(
                    "
                    UPDATE sessions SET otp_code_sent = true WHERE id = $1
                    ",
                )
                .bind(user_session.id)
                .execute(pool.get_ref())
                .await?;

                if let Some(smtp_config) = &config.smtp_config {
                    let db_user = sqlx::query_as::<_, User>(&format!(
                        "
                        SELECT email FROM {} WHERE id = $1
                        ",
                        config.user_table_name
                    ))
                    .bind(user_session.user_id)
                    .fetch_one(pool.get_ref())
                    .await;

                    let otp_code = crate::encryption::decrypt(
                        &user_session.otp_code_encrypted,
                        &format!("{}", user_session.user_id),
                        &config.secret_key,
                    )?;

                    if let Ok(db_user) = db_user {
                        let email = Message::builder()
                            .from(smtp_config.from_email.clone())
                            .to(db_user.email.parse().unwrap())
                            .subject("Your confirmation code")
                            .body(
                                format!(
                                    "
                                Your confirmation code is {}
                                ",
                                    otp_code
                                )
                                .trim()
                                .to_string(),
                            )
                            .unwrap();

                        crate::email::send_email(&config, email)
                    } else if user_session.user_id == INVALID_USER_ID {
                        // Looks like the an attempt to register a duplicate user
                        // There may be a timing attack here.
                    }
                }
            }

            let body = OtpPage {
                hcaptcha: user_session.otp_code_attempts > 0,
                hcaptcha_config: &config.hcaptcha_config,
                errors: &ValidationErrors::default(),
            };

            return Ok(layouts::session_layout(
                "Confirmation Code",
                &body.to_string(),
            ));
        }
    }

    // We shouldn't be here without a session. Go to sign in.
    Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish())
}

pub async fn process_otp(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    session: Option<crate::Session>,
    form: web::Form<Otp>,
) -> Result<HttpResponse, CustomError> {
    if let Some(session) = session {
        if let Some(user_session) = crate::get_user_by_session(&session, pool.get_ref()).await {
            // If we have more than 1 attempt we need to apply the Hcaptcha
            if user_session.otp_code_attempts > 0
                && !super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await
            {
                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                    .finish());
            }

            let otp_code = crate::encryption::decrypt(
                &user_session.otp_code_encrypted,
                &format!("{}", user_session.user_id),
                &config.secret_key,
            )?;

            if otp_code == form.code {
                sqlx::query(
                    "
                    UPDATE sessions 
                    SET otp_code_confirmed = true 
                    AND otp_code_attempts = 0
                    WHERE id = $1
                    ",
                )
                .bind(user_session.id)
                .execute(pool.get_ref())
                .await?;

                if config.auth_type == crate::config::AuthType::Encrypted {
                    return Ok(HttpResponse::SeeOther()
                        .append_header((http::header::LOCATION, crate::DECRYPT_MASTER_KEY_URL))
                        .finish());
                }

                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, config.redirect_url.clone()))
                    .finish());
            } else {
                sqlx::query(
                    "
                    UPDATE 
                        sessions 
                    SET 
                        otp_code_attempts = otp_code_attempts + 1 
                    WHERE 
                        id = $1
                    ",
                )
                .bind(user_session.id)
                .execute(pool.get_ref())
                .await?;

                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, crate::EMAIL_OTP_URL))
                    .finish());
            }
        }
    }

    return Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish());
}

markup::define! {
    OtpPage<'a>(hcaptcha: bool,
    hcaptcha_config: &'a Option<config::HCaptchaConfig>,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Email Confirmation" }

            @forms::TextInput{ title: "Enter your confirmation code", name: "code",
                value:"",
                help_text: "Enter the confirmation code we sent to your email address",
                errors }

            @if let Some(hcaptcha_config) = hcaptcha_config {
                @if *hcaptcha {
                    div."h-captcha"["data-sitekey"=&hcaptcha_config.hcaptcha_site_key] {}
                }
            }

            button.a_button.success[type = "submit"] { "Submit Code" }
        }

        @if let Some(_) = hcaptcha_config {
            @if *hcaptcha {
                script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
            }
        }
    }
}
