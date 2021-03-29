use std::env;


#[derive(Clone, Debug)]
pub struct Config {
    pub webdriver_url: String,
}

impl Config {
    pub fn new() -> Config {


        let webdriver_url: String = if env::var("WEB_DRIVER_URL").is_ok() {
            env::var("WEB_DRIVER_URL").unwrap()
        } else {
            "http://localhost:9515".into()
        };

        Config {
            webdriver_url,
        }
    }
}