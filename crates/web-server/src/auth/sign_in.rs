
use super::super::errors::CustomError;
use crate::config::Config;
use crate::layouts;
use crate::routes::auth::{self, ResetRequest, SignUp};
use crate::{components::forms, config};
use axum::extract::Extension;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub password: String,
    #[serde(rename = "h-captcha-response")]
    pub h_captcha_response: Option<String>,
}

pub async fn sign_in(
    auth::SignIn {}: auth::SignIn,
    Extension(config): Extension<Config>,
) -> Result<impl IntoResponse, CustomError> {
    let body = LoginPage {
        form: &Login::default(),
        hcaptcha_config: &config.hcaptcha_config,
        sign_up: SignUp {}.to_string(),
        reset_password: ResetRequest {}.to_string(),
        errors: &ValidationErrors::default(),
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}


/***pub async fn process_login(
    auth::SignIn {}: auth::SignIn,
    Extension(pool): Extension<Pool>,
    Extension(config): Extension<Config>,
    identity: Identity,
    Form(form): Form<Login>,
) -> Result<impl IntoResponse, CustomError> {
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
        sign_up: SignUp {}.to_string(),
        reset_password: ResetRequest {}.to_string(),
        errors: &validation_errors,
    };

    Ok(layouts::session_layout(
        "Login",
        &body.to_string(),
        config.hcaptcha_config.is_some(),
    ))
}**/

markup::define! {
    LoginPage<'a>(form: &'a  Login,
    hcaptcha_config: &'a Option<config::HCaptchaConfig>,
    sign_up: String, reset_password: String,
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
                a[href=sign_up] { "Sign Up" }
                { " | " }
                a[href=reset_password] { "Reset Password" }
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
