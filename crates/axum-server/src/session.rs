use crate::errors::CustomError;
use axum_extra::extract::CookieJar;
use rand::Rng;
use sha2::{Digest, Sha256};

/***
 * Retrieve the current session from the database based on the cookie.
 * Make sure the cokie has the correct session verifier.
 */

pub async fn get_session(client: &db::Client, jar: &CookieJar, secret_key: Vec<u8> ) -> Option<db::Session> {
    if let Some(session) = jar.get("session") {
        let session = crate::encryption::decrypt(session.value(), "Barricade", &secret_key);

        if let Ok(session) = session {
            let mut split = session.split(":");
            if let (Some(id), Some(verifier)) = (split.next(), split.next()) {
                if let Ok(id) = id.parse::<u64>() {
                    let session_from_db = db::queries::sessions::get_session()
                        .bind(client, &(id as i32))
                        .one()
                        .await
                        .ok();
    
                    return session_from_db;
                }
            }
        }
    }
    None
}

/***
 * Create a very secure cookie that holds our session id and the session verfiier.
 * Encrypt it for defence in depth.
 * Retunr the OTP Code so we can forward it as an email.
 */
pub async fn create_session(
    client: &db::Client,
    email: &str,
    secret_key: Vec<u8>,
) -> Result<(String, u32), CustomError> {
    let (otp_code, otp_code_encrypted) = generate_otp(&secret_key).await?;

    let (session_verifier, hashed_session_verifier) = generate_session_verfier().await;

    let session_id = db::queries::sessions::create_session()
        .bind(
            client,
            &hashed_session_verifier.as_ref(),
            &otp_code_encrypted.as_ref(),
            &email,
        )
        .one()
        .await?;

    let plain_text = format!(
        "{}:{}",
        session_id, session_verifier
    );

    let data_encrypted = crate::encryption::encrypt(&plain_text, "Barricade", &secret_key)?;

    // The cookie has our session_id for fast db lookup, then our
    // sessions verifier so an attacker can't easily forge the session.
    let cookie = format!(
        "session={}; SameSite=Strict; HttpOnly; Secure; Path=/",
        data_encrypted
    );

    Ok((cookie, otp_code))
}

async fn generate_otp(secret: &Vec<u8>) -> Result<(u32, String), CustomError> {
    // We generate and OTP code and encrypt it.
    // Encryption helps secure against an attacker who has read only access to the database
    let mut rng = rand::thread_rng();
    let otp_code: u32 = rng.gen_range(10000..99999);
    let otp_encrypted =
        crate::encryption::encrypt(&format!("{}", otp_code), "Barricade", &secret)?;

    Ok((otp_code, otp_encrypted))
}

async fn generate_session_verfier() -> (String, String) {
    // Create a random session verifier
    let random_bytes = rand::thread_rng().gen::<[u8; 32]>();
    let mut hasher = Sha256::new();
    // Hash it to avoid exposing it in the database. i.e. a passive attacker
    // with access to the database won't be able to re-create the session cookie.
    hasher.update(random_bytes);
    let hashed_session_verifier = hex::encode(hasher.finalize());

    (hex::encode(random_bytes), hashed_session_verifier)
}
