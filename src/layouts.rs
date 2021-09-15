use crate::statics;
use actix_web::{cookie::Cookie, http, HttpResponse};

pub fn redirect_and_snackbar(url: &str, message: &'static str) -> HttpResponse {
    HttpResponse::SeeOther()
        .append_header((http::header::LOCATION, url))
        .cookie(Cookie::build("flash_aargh", message).path("/").finish())
        .finish()
}

markup::define! {
    MarketingHeader <'a>(title: &'a str, csp_content: &'a Option<String>) {

        head {
            meta [ charset="utf-8" ] {}
            meta [ "http-equiv"="X-UA-Compatible", content="IE=edge"] {}
            meta [ name="viewport", content="width=device-width, initial-scale=1" ] {}
            title { {title} }

            script [ src = statics::get_index_js(),
                type="text/javascript", async=""] {}

            link [ rel = "stylesheet", type="text/css" , href = statics::get_index_css()] {}

            @if let Some(csp_content) = {csp_content} {
                meta [ "http-equiv"="Content-Security-Policy", content=csp_content ] {}
            }
        }
    }
    Footer {
        footer.m_footer {
        }
    }
    SnackBar {
        div["data-controller"="notice"] {

        }
    }
    MarketingLayout<'a>(content: &'a str, title: &'a str, csp_content: Option<String>) {

        @markup::doctype()

        html[lang="en"] {

            @MarketingHeader {
                title,
                csp_content
            }

            body.l_marketing {
                #main.container {
                    {markup::raw(content)}
                }
                @Footer {}
            }
            @SnackBar {}
        }
    }

}

pub fn marketing_layout(title: &str, content: &str) -> HttpResponse {
    let csp_content = Some(format!(
        "default-src 'none'; script-src {} 'unsafe-inline'; frame-src {}; style-src {}; connect-src {};",
        "self https://hcaptcha.com https://*.hcaptcha.com",
        "https://hcaptcha.com https://*.hcaptcha.com",
        "self https://hcaptcha.com https://*.hcaptcha.com",
        "https://hcaptcha.com https://*.hcaptcha.com",
    ));

    let l = MarketingLayout {
        title,
        csp_content,
        content,
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(l.to_string())
}

pub fn session_layout(title: &str, content: &str) -> HttpResponse {
    marketing_layout(title, content)
}
