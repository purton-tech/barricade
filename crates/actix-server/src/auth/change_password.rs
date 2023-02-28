use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use validator::{Validate, ValidationErrors};

#[derive(Serialize, Validate, Deserialize, Default)]
pub struct Reset {
    #[validate(length(min = 8, message = "Password is too short"))]
    pub password: String,
    #[validate(must_match(other = "password", message = "The passwords don't match"))]
    pub confirm_password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
    #[validate(length(min = 1, message = "Hidden field"))]
    reset_password_selector: String,
    #[validate(length(min = 1, message = "Hidden field"))]
    reset_password_validator: String,
}

#[derive(Deserialize)]
pub struct Params {
    reset_password_selector: String,
    reset_password_validator: String,
}

pub async fn change_password(
    validator: web::Query<Params>,
    config: web::Data<config::Config>,
) -> Result<HttpResponse> {
    let form = Reset {
        reset_password_selector: validator.reset_password_selector.clone(),
        reset_password_validator: validator.reset_password_validator.clone(),
        ..Default::default()
    };

    let body = ResetPage {
        form: &form,
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout(
        "Change Password",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

pub async fn process_change(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Reset>,
) -> Result<HttpResponse, CustomError> {
    let reset = Reset {
        reset_password_selector: form.reset_password_selector.clone(),
        reset_password_validator: form.reset_password_validator.clone(),
        ..Default::default()
    };

    match form.validate() {
        Ok(_) => {
            let hashed_password = crate::encryption::password_hash(
                &form.password,
                config.use_bcrypt_instead_of_argon,
            )
            .await?;

            let user: Vec<(i32, String)> = sqlx::query_as(&format!(
                "
                SELECT 
                    id, 
                    reset_password_validator_hash 
                FROM {} 
                WHERE 
                    reset_password_selector = $1
                ",
                config.user_table_name
            ))
            .bind(&form.reset_password_selector)
            .fetch_all(pool.get_ref())
            .await?;

            let reset_password_verifier =
                base64::decode_config(&form.reset_password_validator, base64::URL_SAFE)
                    .map_err(|e| CustomError::FaultySetup(e.to_string()))?;
            let reset_password_verifier_hash = Sha256::digest(&reset_password_verifier);

            if let Some(user) = user.first() {
                let reset_password_verifier_hash_from_db =
                    base64::decode_config(&user.1, base64::URL_SAFE)
                        .map_err(|e| CustomError::FaultySetup(e.to_string()))?;

                if compare_constant_time(
                    &reset_password_verifier_hash,
                    &reset_password_verifier_hash_from_db,
                ) {
                    sqlx::query(&format!(
                        "
                        UPDATE {} 
                        SET
                            hashed_password = $1,
                            reset_password_selector = NULL,
                            reset_password_validator_hash = NULL
                        WHERE 
                            id = $2
                        ",
                        config.user_table_name
                    ))
                    .bind(hashed_password)
                    .bind(user.0)
                    .execute(pool.get_ref()) // -> Vec<Person>
                    .await?;
                }
            }

            return Ok(HttpResponse::SeeOther()
                .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
                .finish());
        }
        Err(validation_errors) => {
            let body = ResetPage {
                form: &reset,
                errors: &validation_errors,
            };

            Ok(layouts::session_layout(
                "Change Password",
                &body.to_string(),
                config.hcaptcha_config.is_some(),
            ))
        }
    }
}

pub fn compare_constant_time(x: &[u8], y: &[u8]) -> bool {
    let length = x.len();

    if length != y.len() {
        return false;
    }

    let mut result: u8 = 0;

    for n in 0..length {
        result |= x[n] ^ y[n];
    }

    result == 0
}

markup::define! {
    ResetPage<'a>(form: &'a  Reset,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Change Password" }

            @forms::PasswordInput{ title: "Password", name: "password", value: &form.password, autocomplete: "new-password", help_text: "", errors }
            @forms::PasswordInput{ title: "Confirm Password", name: "confirm_password", value: &form.confirm_password, autocomplete: "new-password", help_text: "", errors }

            input[type="hidden", value=&form.reset_password_selector, name="reset_password_selector"] {}
            input[type="hidden", value=&form.reset_password_validator, name="reset_password_validator"] {}
            button.a_button.success[type = "submit"] { "Change My Password" }
        }
    }
}
