use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use lettre::Message;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Reset {
    pub email: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

pub async fn reset_request(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = ResetPage {
        form: &Reset::default(),
        hcaptcha_config: &config.hcaptcha_config,
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

pub async fn process_request(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Reset>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    if super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await {
        let invitation_selector = rand::thread_rng().gen::<[u8; 8]>();
        let invitation_selector_base64 =
            base64::encode_config(invitation_selector, base64::URL_SAFE);
        let invitation_verifier = rand::thread_rng().gen::<[u8; 24]>();
        let invitation_verifier_hash = Sha256::digest(&invitation_verifier);
        let invitation_verifier_hash_base64 =
            base64::encode_config(invitation_verifier_hash, base64::URL_SAFE);
        let invitation_verifier_base64 =
            base64::encode_config(invitation_verifier, base64::URL_SAFE);

        sqlx::query(&format!(
            "
                UPDATE {} SET 
                    reset_password_selector = $1,
                    reset_password_validator_hash = $2,
                    reset_password_sent_at = now()
                WHERE 
                    email = $3
            ",
            config.user_table_name
        ))
        .bind(&invitation_selector_base64)
        .bind(&invitation_verifier_hash_base64)
        .bind(&form.email.to_lowercase())
        .execute(pool.get_ref()) // -> Vec<Person>
        .await?;

        if let Some(smtp_config) = &config.smtp_config {
            let email = Message::builder()
                .from(smtp_config.from_email.clone())
                .to(form.email.parse().unwrap())
                .subject("Did you request a password reset?")
                .body(
                    format!(
                        "
                        If you requested a password reset please follow this link 
                        \n{}/auth/change_password?reset_password_selector={}&reset_password_validator={}
                        ",
                        smtp_config.domain, invitation_selector_base64, invitation_verifier_base64
                    )
                    .trim()
                    .to_string(),
                )
                .unwrap();

            crate::email::send_email(&config, email)
        }

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

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
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
