use std::env;


#[derive(Clone, Debug)]
pub struct Config {
    pub webdriver_url: String,
    pub headless: bool
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
            headless
        }
    }
}