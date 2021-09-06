use rand::Rng;
use sqlx::PgPool;
use std::env;
use thirtyfour::prelude::*;

#[derive(Clone, Debug)]
pub struct Config {
    pub webdriver_url: String,
    pub host: String,
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

        Config {
            webdriver_url,
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

pub async fn get_otp_code_from_database(config: &Config) -> Result<i32, sqlx::Error> {
    let row: (i32,) =
        sqlx::query_as("SELECT otp_code FROM sessions ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&config.db_pool)
            .await?;

    Ok(row.0)
}

pub fn random_email() -> String {
    let mut rng = rand::thread_rng();
    format!("{}@test.com", rng.gen::<u32>())
}
