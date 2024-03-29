use crate::components::forms;
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::borrow::Cow;
use std::default::Default;
use unicode_normalization::UnicodeNormalization;
use validator::{ValidationError, ValidationErrors};

#[derive(sqlx::FromRow)]
pub struct User {
    id: i32,
    hashed_password: String,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

#[derive(sqlx::FromRow)]
struct InsertedSession {
    id: i32,
}

pub async fn login(config: web::Data<config::Config>) -> Result<HttpResponse> {
    let body = LoginPage {
        form: &Login::default(),
        hcaptcha_config: &config.hcaptcha_config,
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

pub async fn create_session(
    config: &config::Config,
    pool: web::Data<PgPool>,
    identity: Identity,
    user_id: i32,
    master_key_hash: Option<String>,
) -> Result<(), CustomError> {
    // We generate and OTP code and encrypt it.
    // Encryption helps secure against an attacker who has read only access to the database
    let mut rng = rand::thread_rng();
    let otp_code: u32 = rng.gen_range(10000..99999);
    let otp_encrypted = crate::encryption::encrypt(
        &format!("{}", otp_code),
        &format!("{}", user_id),
        &config.secret_key,
    )?;

    // Create a random session verifier
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    let mut hasher = Sha256::new();
    // Hash it to avoid exposing it in the database.
    hasher.update(random_bytes);
    let hex_hashed_session_verifier = hex::encode(hasher.finalize());

    let session = sqlx::query_as::<_, InsertedSession>(
        "
            INSERT INTO sessions (user_id, session_verifier, otp_code_encrypted)
            VALUES($1, $2, $3) RETURNING id
        ",
    )
    .bind(user_id)
    .bind(hex_hashed_session_verifier)
    .bind(otp_encrypted)
    .fetch_one(pool.get_ref())
    .await?;

    let session = crate::Session {
        session_id: session.id,
        session_verifier: hex::encode(random_bytes),
        master_key_hash,
    };

    let serialized =
        serde_json::to_string(&session).map_err(|e| CustomError::FaultySetup(e.to_string()))?;

    identity.remember(serialized);

    Ok(())
}

pub async fn process_login(
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    identity: Identity,
    form: web::Form<Login>,
) -> Result<HttpResponse, CustomError> {
    let mut validation_errors = ValidationErrors::default();

    let valid = super::verify_hcaptcha(&config.hcaptcha_config, &form.h_captcha_response).await;

    if valid {
        let users = sqlx::query_as::<_, User>(&format!(
            "
            SELECT id, hashed_password FROM {} WHERE email = $1
            ",
            config.user_table_name
        ))
        .bind(&form.email.to_lowercase())
        .fetch_all(pool.get_ref()) // -> Vec<Person>
        .await?;

        if !users.is_empty() {
            // Passwords must be normalised
            let normalised_password = &form.password.nfkc().collect::<String>();
            let valid = crate::encryption::verify_hash(
                normalised_password,
                &users[0].hashed_password,
                config.use_bcrypt_instead_of_argon,
            )
            .await?;

            if valid {
                // Generate a session

                create_session(&config, pool, identity, users[0].id, None).await?;

                return Ok(HttpResponse::SeeOther()
                    .append_header((http::header::LOCATION, config.redirect_url.clone()))
                    .finish());
            }
        }

        validation_errors.add(
            "email",
            ValidationError {
                message: Some(Cow::from("Invalid email or password")),
                code: Cow::from("0"),
                params: Default::default(),
            },
        );
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

    let login = Login {
        email: form.email.clone(),
        ..Default::default()
    };

    let body = LoginPage {
        form: &login,
        hcaptcha_config: &config.hcaptcha_config,
        errors: &validation_errors,
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}

markup::define! {
    LoginPage<'a>(form: &'a  Login,
    hcaptcha_config: &'a Option<config::HCaptchaConfig>,
    errors: &'a ValidationErrors) {
        form.m_authentication[id="auth-form", method = "post"] {

            h1 { "Sign In" }

            @forms::EmailInput{ title: "Email", name: "email", value: &form.email, autocomplete: "current-password", help_text: "", errors }
            @forms::PasswordInput{ title: "Password", name: "password", value: &form.password, autocomplete: "current-password", help_text: "", errors }

            @if let Some(hcaptcha_config) = hcaptcha_config {
                button.a_button.success."h-captcha"[
                    "data-sitekey"=&hcaptcha_config.hcaptcha_site_key,
                    "data-callback"="onSubmit"] { "Log In" }
            } else {
                button.a_button.success[type = "submit"] { "Log In" }
            }

            div {
                a[href=crate::SIGN_UP_URL] { "Sign Up" }
                { " | " }
                a[href=crate::RESET_REQUEST_URL] { "Reset Password" }
            }
        }

        @if let Some(_) = hcaptcha_config {
            script[src="https://hcaptcha.com/1/api.js", async="async", defer="defer"] {}
            script[type="text/javascript"] {
                "function onSubmit(token) { document.getElementById('auth-form').submit(); }"
            }
        }
    }
}
