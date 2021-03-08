mod login;
mod registration;

use actix_web::web;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource(crate::SIGN_UP_URL)
            .route(web::get().to(registration::registration))
            .route(web::post().to(registration::process_registration)),
    );
    cfg.service(
        web::resource(crate::SIGN_IN_URL)
            .route(web::get().to(login::login))
            .route(web::post().to(login::process_login)),
    );
    cfg.service(
        web::resource(crate::DECRYPT_MASTER_KEY_URL)
            .route(web::get().to(login::decrypt))
            .route(web::post().to(login::process_decryption)),
    );
    cfg.service(web::resource(crate::SIGN_OUT_URL).route(web::get().to(crate::logout)));
}
