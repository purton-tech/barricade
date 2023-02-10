use actix_web::http::header::{ContentType, Expires};
use actix_web::web::{Path};
use actix_web::{HttpResponse};
use std::time::{Duration,SystemTime};

/// A duration to add to current time for a far expires header.
static FAR: Duration = Duration::from_secs(180 * 24 * 60 * 60);

pub async fn static_file(path: Path<String>) -> HttpResponse {
    let name = &path.into_inner();
    if let Some(data) = assets::files::StaticFile::get(name) {
        let far_expires = SystemTime::now() + FAR;
        HttpResponse::Ok()
            .insert_header(Expires(far_expires.into()))
            .insert_header(ContentType(data.mime.clone()))
            .body(data.content)
    } else {
        HttpResponse::NotFound()
            .reason("No such static file.")
            .finish()
    }
}