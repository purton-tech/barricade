use crate::statics;
use actix_web::{http, HttpResponse};

pub fn redirect_and_snackbar(url: &str, message: &'static str) -> HttpResponse {
    HttpResponse::SeeOther()
        .header(http::header::LOCATION, url)
        .cookie(
            http::Cookie::build("flash_aargh", message)
                .path("/")
                .finish(),
        )
        .finish()
}

markup::define! {
    MarketingHeader <'a>(title: &'a str, csp_content: &'a Option<String>, google_analytics: Option<String>) {

        head {
            meta [ charset="utf-8" ] {}
            meta [ "http-equiv"="X-UA-Compatible", content="IE=edge"] {}
            meta [ name="viewport", content="width=device-width, initial-scale=1" ] {}
            title { {title} }

            //link[rel="icon", type="image/ico",href= format!("/public/{}",statics::favicon_ico.name)]  {}

            script [ src = statics::get_index_js(),
                type="text/javascript", async=""] {}

            link [ rel = "stylesheet", type="text/css" , href = statics::get_index_css()] {}

            @if let Some(ga) = {google_analytics} {
                script[async="async", src=format!("https://www.googletagmanager.com/gtag/js?id={}", ga)] {}
                script {
                    {format!("\
                    window.dataLayer = window.dataLayer || []; \
                    function gtag(){{dataLayer.push(arguments);}} \
                    gtag('js', new Date()); \
                    gtag('config', '{}');", ga)}
                }
            }

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
    MarketingLayout<'a>(content: &'a str, title: &'a str, csp_content: Option<String>,
        google_analytics: Option<String>) {

        @markup::doctype()

        html[lang="en"] {

            @MarketingHeader {
                title,
                csp_content,
                google_analytics: google_analytics.clone()
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

pub fn marketing_layout(
    title: &str,
    content: &str,
    google_analytics: Option<String>,
) -> HttpResponse {
    let l = MarketingLayout {
        title,
        csp_content: None,
        content,
        google_analytics,
    };
    HttpResponse::Ok()
        .content_type("text/html")
        .body(l.to_string())
}

pub fn session_layout(title: &str, content: &str) -> HttpResponse {
    marketing_layout(title, content, None)
}
