use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Reset {
    pub password: String,
    pub confirm_password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

pub async fn change_password() -> Result<HttpResponse> {
    let body = ResetPage {
        form: &Reset::default(),
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_change(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Reset>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    let login = Reset {
        ..Default::default()
    };

    let body = ResetPage {
        form: &login,
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
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Change Password" }

            @forms::PasswordInput{ title: "Password", name: "password", value: &form.password, help_text: "", errors }
            @forms::PasswordInput{ title: "Confirm Password", name: "confirm_password", value: &form.confirm_password, help_text: "", errors }

            button.a_button.success[type = "submit"] { "Change My Password" }
        }
    }
}
