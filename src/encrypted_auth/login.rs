use crate::components::forms::{FormInput, InputType, Stimulus};
use crate::config;
use crate::custom_error::CustomError;
use crate::layouts;
use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::borrow::Cow;
use std::default::Default;
use validator::{ValidationError, ValidationErrors};

#[derive(Serialize, Deserialize, Default)]
pub struct Login {
    pub email: String,
    pub blind_index: String,
}
#[derive(sqlx::FromRow)]
struct User {
    encrypted_private_key: String,
    init_vector: String,
}

#[derive(sqlx::FromRow)]
struct LoginUser {
    id: i32,
}

pub async fn decrypt(
    config: web::Data<config::Config>,
    session: Option<crate::Session>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    // If we have a session cookie, try and convert it to a user.
    let mut logged_user: Option<crate::User> = None;
    if let Some(session) = session {
        logged_user = crate::get_user_by_session_uuid(&session.session_uuid, db_pool.clone()).await;
    }

    if let Some(logged_user) = logged_user {
        let users = sqlx::query_as::<_, User>(&format!(
            "
            SELECT encrypted_private_key, init_vector
            FROM {} WHERE id = $1
            ",
            config.user_table_name
        ))
        .bind(logged_user.user_id)
        .fetch_all(db_pool.get_ref()) // -> Vec<Person>
        .await?;

        let page = DecryptMasterKeyPage {
            init_vector: &users[0].init_vector,
            encrypted_private_key: &users[0].encrypted_private_key,
        };

        return Ok(layouts::session_layout("Master Key", &page.to_string()));
    }

    // We didn't get the session for some reason.
    Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, crate::SIGN_IN_URL))
        .finish())
}

pub async fn process_decryption(config: web::Data<config::Config>) -> Result<HttpResponse> {
    Ok(HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, config.redirect_url.clone()))
        .finish())
}

pub async fn login() -> Result<HttpResponse> {
    let body = LoginPage {
        form: &Login::default(),
        errors: None,
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

pub async fn process_login(
    config: web::Data<config::Config>,
    form: web::Form<Login>,
    identity: Identity,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    let users = sqlx::query_as::<_, LoginUser>(&format!(
        "
        SELECT id FROM {} WHERE email = $1 AND blind_index = $2
        ",
        config.user_table_name
    ))
    .bind(&form.email.to_lowercase())
    .bind(&form.blind_index)
    .fetch_all(db_pool.get_ref()) // -> Vec<Person>
    .await?;

    if !users.is_empty() {
        crate::auth::login::create_session(db_pool, identity, users[0].id).await?;

        return Ok(HttpResponse::SeeOther()
            .append_header((http::header::LOCATION, crate::DECRYPT_MASTER_KEY_URL))
            .finish());
    }

    let mut validation_errors = ValidationErrors::default();

    validation_errors.add(
        "email",
        ValidationError {
            message: Some(Cow::from("Invalid email or password")),
            code: Cow::from("0"),
            params: Default::default(),
        },
    );

    let login = Login {
        email: form.email.clone(),
        ..Default::default()
    };

    let body = LoginPage {
        form: &login,
        errors: Some(validation_errors),
    };

    Ok(layouts::session_layout("Login", &body.to_string()))
}

markup::define! {
    DecryptMasterKeyPage<'a>(encrypted_private_key: &'a str, init_vector: &'a str) {
        .m_decryption["data-controller" = "master"] {
            h1 { "Decrypting Your Master Key" }
            svg.progress[viewBox="0 0 200 200"] {
                path[class="bg", stroke="#ccc", d="M41 149.5a77 77 0 1 1 117.93 0",  fill="none"] {}
                path["data-target" = "master.path", class="meter", stroke="#09c", d="M41 149.5a77 77 0 1 1 117.93 0",
                    fill="none", "stroke-dasharray"="350", "stroke-dashoffset"="350"] {}
            }
            form[method="post", "data-target" = "master.form"] {
                input["data-target" = "master.encryptedPrivateKey", type="hidden", value=encrypted_private_key] {}
                input["data-target" = "master.initVector", type="hidden", value=init_vector] {}
            }
        }
    }
    LoginPage<'a>(form: &'a  Login, errors: Option<ValidationErrors>) {
        div["data-controller" = "login"] {
            form.m_authentication {

                h1 { "Sign In" }

                @FormInput {
                    input_type: InputType::Text,
                    name: String::from("email"),
                    value: String::from(&form.email),
                    label: String::from("Email"),
                    stimulus: Some(Stimulus {
                        data_target: Some("login.email".to_string()),
                        data_action: None
                    }),
                    errors: errors.clone(),
                    ..Default::default()
                }

                @FormInput {
                    input_type: InputType::Password,
                    name: String::from("password"),
                    value: String::from(&form.email),
                    label: String::from("Password"),
                    stimulus: Some(Stimulus {
                        data_target: Some("login.password".to_string()),
                        data_action: None
                    }),
                    ..Default::default()
                }

                button.a_button.success[type = "submit",
                    "data-target" = "login.button",
                    "data-action" = "login#login"] { "Log In" }
                div {
                    a[href=crate::SIGN_UP_URL] { "Sign Up" }
                    { " | " }
                    a[href=crate::SIGN_UP_URL] { "Reset Password" }
                }
            }
            form[method = "post", "data-target" = "login.form"] {
                input[name="email", "data-target" = "login.emailCopy", type="hidden"] {}
                input[name="blind_index", "data-target" = "login.blindIndex", type="hidden"] {}
            }
        }
    }
}
