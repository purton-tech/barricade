pub mod sign_in;
use std::collections::HashSet;

use axum::Router;
use axum_extra::routing::RouterExt;
use serde::{Deserialize, Deserializer};

pub fn routes() -> Router {
    Router::new().typed_get(sign_in::sign_in)
}

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

/***pub async fn verify_hcaptcha(
    hcaptcha_config: &Option<config::HCaptchaConfig>,
    response: &Option<String>,
) -> bool {
    if let Some(hcaptcha) = hcaptcha_config {
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
}**/