use lettre::message;
use std::env;
use std::net::ToSocketAddrs;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub enum AuthType {
    Normal,
    Encrypted,
}
#[derive(Clone, Debug)]
pub struct HCaptchaConfig {
    pub hcaptcha_secret_key: String,
    pub hcaptcha_site_key: String,
}

impl HCaptchaConfig {
    pub fn new() -> Option<HCaptchaConfig> {
        if let Ok(hcaptcha_site_key) = env::var("HCAPTCHA_SITE_KEY") {
            if let Ok(hcaptcha_secret_key) = env::var("HCAPTCHA_SECRET_KEY") {
                Some(HCaptchaConfig {
                    hcaptcha_secret_key,
                    hcaptcha_site_key,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProxyConfig {
    // Configure an optional proxy
    // Who we are proxying
    pub forward_url: Url,
    // Which paths can get through without auth i.e. "^/$,/blog/*,/static/*"
    // in regular expression format, comma seperated.
    pub skip_auth_for: Vec<String>,

    // In proxy mode the maximum size of payload we can receive.
    pub max_payload_size: usize,
}

impl ProxyConfig {
    pub fn new() -> Option<ProxyConfig> {
        let skip_auth_for: Vec<String> = if env::var("SKIP_AUTH_FOR").is_ok() {
            env::var("SKIP_AUTH_FOR")
                .unwrap()
                .split(',')
                .map(|s| s.into())
                .collect()
        } else {
            Default::default()
        };

        let max_payload_size: usize = if env::var("MAX_PAYLOAD_SIZE").is_ok() {
            env::var("MAX_PAYLOAD_SIZE")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        } else {
            2_000_000 // 2mb approx
        };

        if let Ok(forwarded_addr) = env::var("FORWARD_URL") {
            if let Ok(forwarded_port) = env::var("FORWARD_PORT") {
                let forward_port = forwarded_port.parse::<u16>().unwrap();
                let forward_url = Url::parse(&format!(
                    "http://{}",
                    (forwarded_addr, forward_port)
                        .to_socket_addrs()
                        .unwrap()
                        .next()
                        .unwrap()
                ))
                .unwrap();
                return Some(ProxyConfig {
                    forward_url,
                    skip_auth_for,
                    max_payload_size,
                });
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub struct SmtpConfig {
    // Configure SMTP for email.
    pub host: String,
    pub port: u16,
    pub tls_off: bool,
    pub username: String,
    pub password: String,
    pub domain: String,
    pub from_email: message::Mailbox,
}

impl SmtpConfig {
    pub fn new() -> Option<SmtpConfig> {
        if let Ok(host) = env::var("SMTP_HOST") {
            if let Ok(username) = env::var("SMTP_USERNAME") {
                if let Ok(password) = env::var("SMTP_PASSWORD") {
                    if let Ok(smtp_port) = env::var("SMTP_PORT") {
                        if let Ok(domain) = env::var("RESET_DOMAIN") {
                            if let Ok(from_email) = env::var("RESET_FROM_EMAIL_ADDRESS") {
                                Some(SmtpConfig {
                                    host,
                                    port: smtp_port.parse::<u16>().unwrap(),
                                    tls_off: env::var("SMTP_TLS_OFF").is_ok(),
                                    username,
                                    password,
                                    domain,
                                    from_email: from_email.parse().unwrap(),
                                })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    // Which functionality
    pub auth_type: AuthType,
    pub user_table_name: String,
    pub redirect_url: String,
    pub logout_url: String,
    // The database
    pub database_url: String,
    // Id we are using TLS we can set the cookie to secure
    pub secure_cookie: bool,
    // https://www.browserling.com/tools/random-hex
    // And choose 32 bytes (64 digits)
    pub secret_key: Vec<u8>,

    pub hcaptcha_config: Option<HCaptchaConfig>,

    pub proxy_config: Option<ProxyConfig>,

    pub email_otp_enabled: bool,

    pub use_bcrypt_instead_of_argon: bool,

    // Configure SMTP for email.
    pub smtp_config: Option<SmtpConfig>,

    // How many hits oin a fingerprint before we show the captcha
    pub hit_rate: u32,
}

impl Config {
    pub fn new() -> Config {
        let hex = env::var("SECRET_KEY").expect("SECRET_KEY not set");

        let port: u16 = if env::var("PORT").is_ok() {
            env::var("PORT").unwrap().parse::<u16>().unwrap()
        } else {
            9090
        };

        let email_otp_enabled: bool = if env::var("ENABLE_EMAIL_OTP").is_ok() {
            env::var("ENABLE_EMAIL_OTP")
                .unwrap()
                .parse::<bool>()
                .unwrap()
        } else {
            false
        };

        let use_bcrypt_instead_of_argon: bool = env::var("use_bcrypt_instead_of_argon").is_ok();

        let auth_type: AuthType = if env::var("AUTH_TYPE").is_ok() {
            let t = env::var("AUTH_TYPE").unwrap();
            if t.to_lowercase() == "encrypted" {
                AuthType::Encrypted
            } else {
                AuthType::Normal
            }
        } else {
            AuthType::Normal
        };

        let user_table_name: String = if env::var("USER_TABLE_NAME").is_ok() {
            env::var("USER_TABLE_NAME").unwrap()
        } else {
            "users".into()
        };

        let logout_url: String = if env::var("LOGOUT_URL").is_ok() {
            env::var("LOGOUT_URL").unwrap()
        } else {
            "/".into()
        };

        Config {
            port,
            auth_type,
            user_table_name,
            redirect_url: env::var("REDIRECT_URL").expect("REDIRECT_URL not set"),
            logout_url,
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
            secure_cookie: env::var("SECURE_COOKIE").is_ok(),
            secret_key: hex_to_bytes(&hex).expect("SECRET_KEY could not parse"),
            hcaptcha_config: HCaptchaConfig::new(),
            proxy_config: ProxyConfig::new(),
            email_otp_enabled,
            use_bcrypt_instead_of_argon,
            smtp_config: SmtpConfig::new(),
            hit_rate: 10,
        }
    }
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect()
}
