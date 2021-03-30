use std::env;
use thirtyfour::prelude::*;
use rand::Rng;

#[derive(Clone, Debug)]
pub struct Config {
    pub webdriver_url: String,
    pub headless: bool,
}

impl Config {
    pub fn new() -> Config {
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

        Config {
            webdriver_url,
            headless,
        }
    }

    pub async fn get_driver(&self) -> WebDriverResult<WebDriver> {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("--no-sandbox")?;
        caps.add_chrome_arg("--disable-gpu")?;
        if self.headless {
            caps.set_headless()?;
        }
        WebDriver::new(&self.webdriver_url, &caps).await
    }
}

pub fn random_email() -> String {
    
    let mut rng = rand::thread_rng();
    format!("{}", rng.gen::<u32>())
}
