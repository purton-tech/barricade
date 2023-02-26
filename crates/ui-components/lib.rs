mod layout;
pub mod sign_in;
pub mod email_otp;
pub mod encryption_login;
pub mod encryption_registration;

pub static SIGN_IN: &str = "/auth/sign_in";
pub static EMAIL_OTP: &str = "/auth/email_otp";
pub static ENCRYPTION_PASSWORD: &str = "/auth/encryption_password";