use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/auth/encryption_password", get(encryption_password))
        .route(
            "/auth/encryption_password",
            post(process_encryption_password),
        )
}

pub async fn encryption_password(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar
) -> Result<Html<String>, CustomError> {

    let client = pool.get().await?;

    let session_from_db =
        crate::session::get_session(&client, &jar, config.secret_key.clone()).await;

    if let Some(session) = session_from_db {
        if let Some(user_id) = session.user_id {

            let keys = db::queries::encryption_keys::get_user_keys()
                .bind(&client, &user_id)
                .all()
                .await?;

            if keys.len() > 0 {
                return Ok(Html(
                    ui_components::encryption_login::encryption_password(),
                ));

            }
        }
    }    

    Ok(Html(
        ui_components::encryption_registration::encryption_password(),
    ))
}

#[derive(Deserialize, Default)]
pub struct KeyGeneration {
    pub master_password_hash: String,
    pub protected_symmetric_key: String,

    // ECDSA
    pub protected_ecdsa_private_key: String,
    pub ecdsa_public_key: String,

    // ECDH
    pub protected_ecdh_private_key: String,
    pub ecdh_public_key: String,
}

pub async fn process_encryption_password(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar,
    Form(keygen_form): Form<KeyGeneration>,
) -> Result<impl IntoResponse, CustomError> {
    let client = pool.get().await?;

    Ok(Redirect::to(&config.redirect_url))
}
