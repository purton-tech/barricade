mod change_password;
mod email_otp;
pub mod login;
mod registration;
mod reset_request;
use crate::config;
use actix_web::web;
use reqwest::Url;
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Code {
    MissingSecret,
    InvalidSecret,
    MissingResponse,
    InvalidResponse,
    BadRequest,
    Unknown(String),
}

impl<'de> Deserialize<'de> for Code {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = String::deserialize(de)?;
        Ok(match &*code {
            "missing-input-secret" => Code::MissingSecret,
            "invalid-input-secret" => Code::InvalidSecret,
            "missing-input-response" => Code::MissingResponse,
            "invalid-input-response" => Code::InvalidResponse,
            "bad-request" => Code::BadRequest,
            _ => Code::Unknown(code),
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct RecaptchaResponse {
    pub success: bool,
    #[serde(rename = "error-codes")]
    pub error_codes: Option<HashSet<Code>>,
}

pub async fn verify_hcaptcha(
    hcaptcha_config: &Option<config::HCaptchaConfig>,
    response: &Option<String>,
) -> bool {
    // If we are jkust testing ignore the hcaptcha.
    // It was causing issues in the browser testing.
    if let Some(hcaptcha) = hcaptcha_config {
        //if hcaptcha.hcaptcha_secret_key == "0x0000000000000000000000000000000000000000" {
        //    return true;
        //}

        if let Some(resp) = response {
            let mut url = Url::parse("https://hcaptcha.com/siteverify").unwrap();

            let secret_key: &str = &hcaptcha.hcaptcha_secret_key;

            url.query_pairs_mut()
                .extend_pairs(&[("secret", secret_key), ("response", resp)]);

            let response = reqwest::get(url).await;

            if let Ok(resp) = response {
                let recaptcha_response = resp.json::<RecaptchaResponse>().await;

                if let Ok(status) = recaptcha_response {
                    if status.success {
                        return true;
                    } else {
                        dbg!(status);
                    }
                }
            }
        }

        return false;
    }

    true
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(crate::SIGN_UP_URL)
            .route(web::get().to(registration::registration))
            .route(web::post().to(registration::process_registration)),
    );
    cfg.service(
        web::resource(crate::SIGN_IN_URL)
            .route(web::get().to(login::login))
            .route(web::post().to(login::process_login)),
    );
    cfg.service(web::resource(crate::SIGN_OUT_URL).route(web::get().to(crate::logout)));
    cfg.service(
        web::resource(crate::RESET_REQUEST_URL)
            .route(web::get().to(reset_request::reset_request))
            .route(web::post().to(reset_request::process_request)),
    );
    cfg.service(
        web::resource(crate::CHANGE_PASSWORD_URL)
            .route(web::get().to(change_password::change_password))
            .route(web::post().to(change_password::process_change)),
    );
    cfg.service(
        web::resource(crate::EMAIL_OTP_URL)
            .route(web::get().to(email_otp::email_otp))
            .route(web::post().to(email_otp::process_otp)),
    );
}
