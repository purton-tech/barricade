mod config;
mod errors;
mod sign_in;
mod email_otp;
mod reverse_proxy;

use axum::{extract::Extension, Router};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower::{make::Shared, ServiceExt};

#[tokio::main]
async fn main() {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "barricade=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // build our application with a route
    let app = Router::new()
        .merge(sign_in::routes())
        .merge(email_otp::routes())
        .layer(Extension(config))
        .layer(Extension(pool.clone()));
        // Add auth layer
        // Add reverse proxy layer

    let service = tower::service_fn(move |req: hyper::Request<hyper::Body> | {
        let router_svc = app.clone();
        async move {
            if req.uri() == "/" {
                reverse_proxy::proxy(req).await
            } else {
                router_svc.oneshot(req).await.map_err(|err| match err {})
            }
        }
    });
    

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(Shared::new(service))
        .await
        .unwrap();
}
