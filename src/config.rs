use aes_gcm::aead::{generic_array::GenericArray, Aead, NewAead, Payload};
use aes_gcm::AeadInPlace;
use aes_gcm::Aes256Gcm; // Or `Aes128Gcm`
use argon2::password_hash::rand_core::RngCore;
use lettre::message;
use rand_core::OsRng;
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
    // The database
    pub database_url: String,
    // Id we are using TLS we can set the cookie to secure
    pub secure_cookie: bool,
    // https://www.browserling.com/tools/random-hex
    // And choose 32 bytes (64 digits)
    pub secret_key: Vec<u8>,
    // Who we are proxying
    pub forward_url: Url,
    // Which paths can get through without auth i.e. "^/$,/blog/*,/static/*"
    // in regular expression format, comma seperated.
    pub skip_auth_for: Vec<String>,
    pub hcaptcha_config: Option<HCaptchaConfig>,

    pub email_otp_enabled: bool,

    pub use_bcrypt_instead_of_argon: bool,

    // Configure SMTP for email.
    pub smtp_config: Option<SmtpConfig>,

    // How many hits oin a fingerprint before we show the captcha
    pub hit_rate: u32,

    // In proxy mode the maximum size of payload we can receive.
    pub max_payload_size: usize,
}

impl Config {
    pub fn new() -> Config {
        let hex = env::var("SECRET_KEY").expect("SECRET_KEY not set");
        let forwarded_addr = env::var("FORWARD_URL").expect("FORWARD_URL not set");
        let forwarded_port = env::var("FORWARD_PORT")
            .expect("FORWARD_PORT not set")
            .parse::<u16>()
            .unwrap();

        let forward_url = Url::parse(&format!(
            "http://{}",
            (forwarded_addr, forwarded_port)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap()
        ))
        .unwrap();

        let skip_auth_for: Vec<String> = if env::var("SKIP_AUTH_FOR").is_ok() {
            env::var("SKIP_AUTH_FOR")
                .unwrap()
                .split(',')
                .map(|s| s.into())
                .collect()
        } else {
            Default::default()
        };

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

        let use_bcrypt_instead_of_argon: bool = if env::var("use_bcrypt_instead_of_argon").is_ok() {
            true
        } else {
            false
        };

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

        let max_payload_size: usize = if env::var("MAX_PAYLOAD_SIZE").is_ok() {
            env::var("MAX_PAYLOAD_SIZE")
                .unwrap()
                .parse::<usize>()
                .unwrap()
        } else {
            2_000_000 // 2mb approx
        };

        Config {
            port,
            auth_type,
            user_table_name,
            redirect_url: env::var("REDIRECT_URL").expect("REDIRECT_URL not set"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL not set"),
            secure_cookie: env::var("SECURE_COOKIE").is_ok(),
            secret_key: hex_to_bytes(&hex).expect("SECRET_KEY could not parse"),
            forward_url,
            skip_auth_for,
            hcaptcha_config: HCaptchaConfig::new(),
            email_otp_enabled,
            use_bcrypt_instead_of_argon,
            smtp_config: SmtpConfig::new(),
            hit_rate: 10,
            max_payload_size,
        }
    }

    /// Encrypts the plain text with authenticated encryption providing
    /// confidentiality, integrity, and authenticity.
    pub fn encrypt(&self, plain_text: &str, aad: &str) -> Result<String, crate::CustomError> {
        // Create a vec to hold the [nonce | cookie value | tag].
        let val = plain_text.as_bytes();
        let mut data = vec![0; NONCE_LEN + val.len() + TAG_LEN];

        // Split data into three: nonce, input/output, tag. Copy input.
        let (nonce, in_out) = data.split_at_mut(NONCE_LEN);
        let (in_out, tag) = in_out.split_at_mut(val.len());
        in_out.copy_from_slice(val);

        // Fill nonce piece with random data.
        OsRng.fill_bytes(nonce);
        let nonce = GenericArray::clone_from_slice(nonce);

        // Perform the actual sealing operation, using aad as
        // associated data to prevent value swapping.
        let aad = aad.as_bytes();
        let aead = Aes256Gcm::new(GenericArray::from_slice(&self.secret_key));
        let aad_tag = aead
            .encrypt_in_place_detached(&nonce, aad, in_out)
            .expect("encryption failure!");

        // Copy the tag into the tag piece.
        tag.copy_from_slice(&aad_tag);

        // Base64 encode [nonce | encrypted value | tag].
        Ok(base64::encode(&data))
    }

    /// Given a encrypted value `str` and an aad, where the nonce is
    /// prepended to the original value and then both are Base64 encoded,
    /// verifies and decrypts the sealed value and returns it. If there's a
    /// problem, returns an `Err` with a string describing the issue.
    pub fn decrypt(&self, cipher: &str, aad: &str) -> Result<String, crate::CustomError> {
        let data = base64::decode(cipher)
            .map_err(|_| crate::CustomError::FaultySetup("bad base64 value".into()))?;
        if data.len() <= NONCE_LEN {
            return Err(crate::CustomError::FaultySetup(
                "length of decoded data is <= NONCE_LEN".into(),
            ));
        }

        let (nonce, cipher) = data.split_at(NONCE_LEN);
        dbg!(&nonce);
        let payload = Payload {
            msg: cipher,
            aad: aad.as_bytes(),
        };

        let aead = Aes256Gcm::new(GenericArray::from_slice(&self.secret_key));
        let decrypted = aead
            .decrypt(GenericArray::from_slice(nonce), payload)
            .map_err(|e| crate::CustomError::FaultySetup(e.to_string()))?;

        let decrypted = String::from_utf8(decrypted)
            .map_err(|e| crate::CustomError::FaultySetup(e.to_string()))?;

        Ok(decrypted)
    }
}

pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const TAG_LEN: usize = 16;
pub(crate) const KEY_LEN: usize = 32;

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption() {
        let config = Config::new();
        let cipher_text = config.encrypt("Hello World", "AAD").unwrap();

        let plain_text = config.decrypt(&cipher_text, "AAD").unwrap();

        assert_eq!(plain_text, "Hello World");
    }
}
