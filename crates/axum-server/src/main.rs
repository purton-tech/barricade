mod config;
mod email_otp;
mod encryption;
mod errors;
mod reverse_proxy;
mod sign_in;
mod static_files;
mod encryption_password;

use axum::{
    extract::Extension,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_extra::extract::CookieJar;
use config::Config;
use hyper::HeaderMap;
use std::net::SocketAddr;
use tower::{make::Shared, ServiceExt};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "barricade=trace,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // All the env vars a hekld in a config struct.
    let config = Config::new();

    let pool = db::create_pool(&config.database_url);

    // build our application with a route
    let app = Router::new()
        .merge(sign_in::routes())
        .merge(email_otp::routes())
        .merge(encryption_password::routes())
        .route("/auth/static/*path", get(static_files::static_path))
        .layer(Extension(config.clone()))
        .layer(Extension(pool.clone()));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    if let Some(proxy_config) = config.proxy_config {
        // This is based on the https://github.com/tokio-rs/axum/tree/main/examples/http-proxy
        // Basically any non /auth routes get forwarded to the application but only
        // if those routes are authenticated.
        let service = tower::service_fn(move |req: hyper::Request<hyper::Body>| {
            let router_svc = app.clone();
            let skip_auth_for = proxy_config.skip_auth_for.clone();
            let pool_copy = pool.clone();
            async move {
                // If we are /auth then barricade handles the request
                if req.uri().to_string().starts_with("/auth") {
                    router_svc.oneshot(req).await.map_err(|err| match err {})
                } else {
                    if is_authenticated(req.headers(), &skip_auth_for, pool_copy).await {
                        // For the request to get forwarded to the actual application
                        // the following need to be true
                        //
                        // We have a valid session, or
                        // The path is in the ignore list.
                        reverse_proxy::proxy(req).await
                    } else {
                        Ok(Redirect::to(ui_components::SIGN_IN).into_response())
                    }
                }
            }
        });

        // run it
        axum::Server::bind(&addr)
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .serve(Shared::new(service))
            .await
            .unwrap();
    } else {
        // Run without the proxy.
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

/***
 * If the user has a valid session or the route they want to get to is in the
 * list of routes that don't require auith then they are good to go.
 *
 * Otherwise false.
 */
async fn is_authenticated(
    headers: &HeaderMap,
    skip_auth_for: &Vec<String>,
    pool: db::Pool,
) -> bool {
    let jar = CookieJar::from_headers(headers);

    if let Some((id, verifier)) = crate::email_otp::get_session(&jar).await {
        let client = pool.get().await;

        if let Ok(client) = client {
            let session_from_db = db::queries::sessions::get_session()
                .bind(&client, &(id as i32))
                .one()
                .await;

            if let Ok(session_from_db) = session_from_db {
                dbg!(session_from_db);

                return true;
            }
        }
    }

    false
}
