use crate::errors::CustomError;
use axum::{
    extract::Extension,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use axum_extra::extract::CookieJar;

pub fn routes() -> Router {
    Router::new()
        .route(ui_components::DECRYPT_KEYS, get(decrypt))
        .route(ui_components::DECRYPT_KEYS, post(process_decrypt))
}

/***
 * Pull the users keys from the database and pass them to the front end for the decryption
 * phhase.
 */
pub async fn decrypt(
    Extension(config): Extension<crate::config::Config>,
    Extension(pool): Extension<db::Pool>,
    jar: CookieJar,
) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;

    let session_from_db =
        crate::session::get_session(&client, &jar, config.secret_key.clone()).await;

    if let Some(session) = session_from_db {
        if let Some(user_id) = session.user_id {
            let key = db::queries::encryption_keys::get_user_keys()
                .bind(&client, &user_id)
                .one()
                .await?;

            return Ok(Html(ui_components::decrypt_keys::decrypt(
                ui_components::decrypt_keys::Props {
                    protected_symmetric_key: key.protected_symmetric_key,
                    public_ecdh_key: key.ecdh_public_key,
                    protected_ecdh_private_key: key.protected_ecdh_private_key,
                    public_ecdsa_key: key.ecdsa_public_key,
                    protected_ecdsa_private_key: key.protected_ecdsa_private_key,
                    email: session.email,
                },
            )));
        }
    }

    Err(CustomError::FaultySetup(
        "Problem setting up key decryption".to_string(),
    ))
}

pub async fn process_decrypt(
    Extension(config): Extension<crate::config::Config>
) -> Result<impl IntoResponse, CustomError> {
    Ok(Redirect::to(&config.redirect_url))
}
