use actix_identity::Identity;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    cookie, dev::Payload, http, middleware, web, App, Error, FromRequest, HttpRequest,
    HttpResponse, HttpServer,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};
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
pub static RESET_REQUEST_URL: &str = "/auth/reset_request";
pub static EMAIL_OTP_URL: &str = "/auth/email_otp";
pub static CHANGE_PASSWORD_URL: &str = "/auth/change_password/{reset_token}";

pub static COOKIE_NAME: &str = "session";
pub static USER_HEADER_NAME: &str = "x-user-id";

#[derive(sqlx::FromRow, Debug)]
struct User {
    user_id: i32,
    otp_code_confirmed: bool,
}

pub async fn logout(
    id: Identity,
    session: Option<Session>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CustomError> {
    if let Some(session) = session {
        if let Ok(uuid) = Uuid::parse_str(&session.session_uuid) {
            sqlx::query(
                "
                DELETE FROM sessions WHERE session_uuid = $1
                ",
            )
            .bind(uuid)
            .execute(pool.get_ref()) // -> Vec<Person>
            .await?;
        }
    }
    id.forget();

    Ok(crate::layouts::redirect_and_snackbar(
        "/",
        "You succesfully logged out",
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub session_uuid: String,
}

impl FromRequest for Session {
    type Config = ();
    type Error = CustomError;
    type Future = Ready<Result<Session, CustomError>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Some(session_uuid) = identity.identity() {
                return ok(Session { session_uuid });
            }
        }
        err(CustomError::Unauthorized)
    }
}

async fn authorize(
    req: HttpRequest,
    session: Option<Session>,
    body: web::Bytes,
    config: web::Data<config::Config>,
    pool: web::Data<PgPool>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    // If we have a session cookie, try and convert it to a user.
    let mut logged_user: Option<User> = None;
    if let Some(session) = session {
        logged_user = get_user_by_session_uuid(&session.session_uuid, pool).await;
    }

    // If we have Email Otp make sure the user has entered the code
    if let Some(logged_user) = &logged_user {
        if config.email_otp_enabled && logged_user.otp_code_confirmed == false {
            return Ok(HttpResponse::SeeOther()
                .append_header((http::header::LOCATION, SIGN_IN_URL))
                .finish());
        }
    }

    // If the contor header is set, then this is an envoy external auth request.
    if let Some(_header) = req.headers().get("x-envoy-internal") {
        envoy_external_auth(logged_user).await
    } else {
        reverse_proxy(req, logged_user, body, config, client).await
    }
}

async fn envoy_external_auth(logged_user: Option<User>) -> Result<HttpResponse, Error> {
    if let Some(logged_user) = logged_user {
        let mut resp = HttpResponse::Ok();
        resp.append_header((USER_HEADER_NAME, format!("{:?}", logged_user.user_id)));
        Ok(resp.finish())
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}

async fn get_user_by_session_uuid(session_uuid: &str, pool: web::Data<PgPool>) -> Option<User> {
    if let Ok(uuid) = Uuid::parse_str(session_uuid) {
        let user = sqlx::query_as::<_, User>(
            "
            SELECT user_id, otp_code_confirmed FROM sessions WHERE session_uuid = $1
            ",
        )
        .bind(uuid)
        .fetch_one(pool.get_ref()) // -> Vec<Person>
        .await;

        if let Ok(user) = user {
            return Some(user);
        }
    }

    None
}

async fn reverse_proxy(
    req: HttpRequest,
    logged_user: Option<User>,
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
            forwarded_req.append_header((USER_HEADER_NAME, format!("{:?}", logged_user.user_id)))
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
            .service(statics::static_file)
            .configure(auth_routes)
            // The proxy
            .default_service(web::route().to(authorize))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&config.secret_key)
                    .name(COOKIE_NAME)
                    .path("/")
                    .same_site(cookie::SameSite::Strict)
                    .http_only(true)
                    .secure(config.secure_cookie), // If we are using ssl the set the cookie to secure.
            ))
            // enable logger - always register actix-web Logger middleware last
            .wrap(middleware::Logger::default())
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
