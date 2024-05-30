use super::super::errors::CustomError;
use crate::routes::routes::auth;
use axum::extract::Extension;
use axum::response::Html;
use db::Pool;

pub async fn index(
    auth::Index {}: auth::Index,
    Extension(_pool): Extension<Pool>,
) -> Result<Html<String>, CustomError> {
    Ok(Html("<h1>Hello</h1>".to_string()))
}
