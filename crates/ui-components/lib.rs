pub mod decrypt_keys;
pub mod email_otp;
pub mod encryption_login;
pub mod encryption_registration;
mod layout;
pub mod sign_in;

pub static SIGN_IN: &str = "/auth/sign_in";
pub static EMAIL_OTP: &str = "/auth/email_otp";
pub static ENCRYPTION_PASSWORD: &str = "/auth/encryption_password";
pub static ENCRYPTION_PASSWORD_LOGON: &str = "/auth/encryption_password_logon";
pub static DECRYPT_KEYS: &str = "/auth/decrypt_keys";