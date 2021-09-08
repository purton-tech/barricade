use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead, Payload};
use aes_gcm::Aes256Gcm;
use rand::Rng;
use sqlx::PgPool;
use std::env;
use thirtyfour::prelude::*; // Or `Aes128Gcm`

#[derive(Clone, Debug)]
pub struct Config {
    pub webdriver_url: String,
    pub host: String,
    pub secret_key: Vec<u8>,
    // The database
    pub db_pool: PgPool,
    pub headless: bool,
}

impl Config {
    pub async fn new() -> Config {
        let webdriver_url: String = if env::var("WEB_DRIVER_URL").is_ok() {
            env::var("WEB_DRIVER_URL").unwrap()
        } else {
            // Default to selenium in our dev container
            "http://selenium:4444/wd/hub".into()
        };

        let headless = if env::var("DISABLE_HEADLESS").is_ok() {
            false
        } else {
            true
        };

        let host = if env::var("WEB_DRIVER_DESTINATION_HOST").is_ok() {
            env::var("WEB_DRIVER_DESTINATION_HOST").unwrap()
        } else {
            "http://development:9095".into()
        };

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let db_pool = PgPool::connect(&database_url).await.unwrap();

        let hex = env::var("SECRET_KEY").expect("SECRET_KEY not set");
        Config {
            webdriver_url,
            secret_key: hex_to_bytes(&hex).expect("SECRET_KEY could not parse"),
            host,
            db_pool,
            headless,
        }
    }

    pub async fn get_driver(&self) -> WebDriverResult<WebDriver> {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("--no-sandbox")?;
        caps.add_chrome_arg("--disable-gpu")?;
        // We need the below otherwise window.crypto.subtle is not defined
        caps.add_chrome_arg("--unsafely-treat-insecure-origin-as-secure=http://development:9095")?;

        if self.headless {
            caps.set_headless()?;
        }
        WebDriver::new(&self.webdriver_url, &caps).await
    }
}

pub(crate) const NONCE_LEN: usize = 12;

pub fn decrypt(cipher: &str, aad: &str, secret_key: &[u8]) -> String {
    let data = base64::decode(cipher).unwrap();

    let (nonce, cipher) = data.split_at(NONCE_LEN);
    dbg!(&nonce);
    let payload = Payload {
        msg: cipher,
        aad: aad.as_bytes(),
    };

    let aead = Aes256Gcm::new(GenericArray::from_slice(secret_key));
    let decrypted = aead
        .decrypt(GenericArray::from_slice(nonce), payload)
        .unwrap();

    let decrypted = String::from_utf8(decrypted).unwrap();

    decrypted
}

pub async fn get_otp_code_from_database(config: &Config) -> Result<String, sqlx::Error> {
    let row: (String, i32) = sqlx::query_as(
        "SELECT otp_code_encrypted, user_id FROM sessions ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_one(&config.db_pool)
    .await?;

    let decrypted_otp = decrypt(&row.0, &format!("{}", row.1), &config.secret_key);

    Ok(decrypted_otp)
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect()
}

pub fn random_email() -> String {
    let mut rng = rand::thread_rng();
    format!("{}@test.com", rng.gen::<u32>())
}
