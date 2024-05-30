// All the routes of the application are mapped here and are typesafe
// https://docs.rs/axum-extra/latest/axum_extra/routing/trait.TypedPath.html
pub mod routes {

    pub mod auth {
        use axum_extra::routing::TypedPath;
        use serde::Deserialize;

        #[derive(TypedPath, Deserialize)]
        #[typed_path("/auth/sign_in")]
        pub struct Index {}
    }
}
