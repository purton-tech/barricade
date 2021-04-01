use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    cookie, dev::Payload, http, middleware, web, App, Error, FromRequest, HttpRequest,
    HttpResponse, HttpServer,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::io;
mod components;
mod config;
mod custom_error;
mod layouts;
use awc::Client;
use custom_error::CustomError;
use futures::future::{err, ok, Ready};

pub mod statics {
    include!(concat!(env!("OUT_DIR"), "/statics.rs"));
}

mod auth;
mod encrypted_auth;

pub static SIGN_UP_URL: &str = "/auth/sign_up";
pub static SIGN_IN_URL: &str = "/auth/sign_in";
pub static DECRYPT_MASTER_KEY_URL: &str = "/auth/decrypt";
pub static SIGN_OUT_URL: &str = "/auth/sign_out";

pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();

    crate::layouts::redirect_and_snackbar("/", "You succesfully logged out")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggedUser {
    pub id: i32,
}

impl FromRequest for LoggedUser {
    type Config = ();
    type Error = CustomError;
    type Future = Ready<Result<LoggedUser, CustomError>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Some(user_json) = identity.identity() {
                if let Ok(user) = serde_json::from_str(&user_json) {
                    return ok(user);
                }
            }
        }
        err(CustomError::Unauthorized)
    }
}

async fn forward(
    req: HttpRequest,
    logged_user: Option<LoggedUser>,
    body: web::Bytes,
    config: web::Data<config::Config>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let mut new_url = config.forward_url.clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    let bypass_auth = config.skip_auth_for.iter().any(|path| {
        let re = regex::Regex::new(path).unwrap();
        re.is_match(req.uri().path())
    });

    if bypass_auth || logged_user.is_some() {
        // TODO: This forwarded implementation is incomplete as it only handles the inofficial
        // X-Forwarded-For header but not the official Forwarded one.
        let forwarded_req = client
            .request_from(new_url.as_str(), req.head())
            .no_decompress();
        let forwarded_req = if let Some(addr) = req.head().peer_addr {
            forwarded_req.append_header(("x-forwarded-for", format!("{}", addr.ip())))
        } else {
            forwarded_req
        };

        // Add the user id as a header.
        let fwd_req = if let Some(logged_user) = logged_user {
            forwarded_req.append_header(("user", format!("{:?}", logged_user.id)))
        } else {
            forwarded_req
        };

        let mut res = fwd_req.send_body(body).await.map_err(Error::from)?;

        let mut client_resp = HttpResponse::build(res.status());
        // Remove `Connection` as per
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Connection#Directives
        for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection")
        {
            client_resp.append_header((header_name.clone(), header_value.clone()));
        }

        Ok(client_resp.body(res.body().await?))
    } else {
        return Ok(HttpResponse::SeeOther()
            .append_header((http::header::LOCATION, SIGN_IN_URL))
            .finish());
    }
}

async fn envoy_external_auth(req: HttpRequest) -> Result<HttpResponse, Error> {
    dbg!(req);
    Ok(HttpResponse::Ok().finish())
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = config::Config::new();
    let port = config.port;
    let db_pool = PgPool::connect(&config.database_url).await.unwrap();

    let auth_routes = if config.auth_type == config::AuthType::Normal {
        auth::routes
    } else {
        encrypted_auth::routes
    };

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone()) // pass database pool to application so we can access it inside handlers
            .data(config.clone())
            .data(Client::new())
            .service(web::resource("/").route(web::get().to(envoy_external_auth)))
            .service(statics::static_file)
            .configure(auth_routes)
            // The proxy
            .default_service(web::route().to(forward))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&config.secret_key)
                    .name("auth")
                    .path("/")
                    .same_site(cookie::SameSite::Strict)
                    .secure(config.secure_cookie), // If we are using ssl the set the cookie to secure.
            ))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
