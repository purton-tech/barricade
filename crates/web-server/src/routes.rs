// All the routes of the application are mapped here and are typesafe
// https://docs.rs/axum-extra/latest/axum_extra/routing/trait.TypedPath.html
pub mod auth {
    use axum_extra::routing::TypedPath;
    use serde::Deserialize;

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/sign_in")]
    pub struct SignIn {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/sign_up")]
    pub struct SignUp {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/sign_out")]
    pub struct SignOut {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/reset_request")]
    pub struct ResetRequest {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/email_otp")]
    pub struct EmailOtp {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/change_password")]
    pub struct ChnagePassword {}

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/auth/decrypt")]
    pub struct Decrypt {}
}
