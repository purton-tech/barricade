use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use std::default::Default;
use validator::{Validate, ValidationErrors};

#[derive(Serialize, Validate, Deserialize, Default)]
pub struct Reset {
    #[validate(length(min = 8, message = "Password is too short"))]
    pub password: String,
    #[validate(must_match(other = "password", message = "The passwords don't match"))]
    pub confirm_password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(Deserialize)]
pub struct Params {
    reset_token: String,
}

pub async fn change_password() -> Result<HttpResponse> {
    let body = ResetPage {
        form: &Reset::default(),
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_change(
    info: web::Path<Params>,
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Reset>,
) -> Result<HttpResponse, CustomError> {
    let reset = Reset {
        ..Default::default()
    };

    match form.validate() {
        Ok(_) => {
            let hashed_password =
                hash(&form.password, DEFAULT_COST).map_err(|_| CustomError::Unauthorized)?;

            if let Ok(uuid) = Uuid::parse_str(&info.reset_token) {
                dbg!(&uuid);

                sqlx::query(&format!(
                    "
                        UPDATE {} 
                            SET hashed_password = $1,
                            reset_password_token = NULL,
                            reset_password_sent_at = NULL
                        WHERE 
                            reset_password_token = $2
                        AND 
                            reset_password_sent_at > now() - '1 hour'::interval
                    ",
                    config.user_table_name
                ))
                .bind(hashed_password)
                .bind(uuid)
                .execute(pool.get_ref())
                .await?;
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

            Ok(layouts::session_layout("Registration", &body.to_string()))
        }
    }
}

markup::define! {
    ResetPage<'a>(form: &'a  Reset,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Change Password" }

            @forms::PasswordInput{ title: "Password", name: "password", value: &form.password, help_text: "", errors }
            @forms::PasswordInput{ title: "Confirm Password", name: "confirm_password", value: &form.confirm_password, help_text: "", errors }

            button.a_button.success[type = "submit"] { "Change My Password" }
        }
    }
}
