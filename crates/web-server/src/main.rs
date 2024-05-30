mod auth;
mod config;
mod errors;
mod routes;
mod static_files;
use axum::{Extension, Router};
use axum_extra::routing::RouterExt;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = config::Config::new();
    let pool = db::create_pool(&config.database_url);
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // build our application with a route
    let app = Router::new()
        .typed_get(static_files::static_path)
        .merge(auth::routes())
        .layer(Extension(config))
        .layer(Extension(pool.clone()));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
