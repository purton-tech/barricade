use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_web::{http, web, HttpResponse, Result};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Otp {
    pub code: String,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Session {
    otp_code: i32,
    otp_code_attempts: i32,
}

pub async fn email_otp(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = OtpPage {
        form: &Otp::default(),
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_otp(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    form: web::Form<Otp>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    return Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish());
}

markup::define! {
    OtpPage<'a>(form: &'a  Otp,
    errors: &'a ValidationErrors) {
        form.m_authentication[method = "post"] {

            h1 { "Password Reset Request" }

            @forms::NumberInput{ title: "Code", name: "code", value: &form.code, help_text: "", errors }

            button.a_button.success[type = "submit"] { "Submit Code" }
        }
    }
}
