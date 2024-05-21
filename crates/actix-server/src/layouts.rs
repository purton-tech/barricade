use actix_web::{http, HttpResponse};
use assets::files;

markup::define! {
    Header <'a>(title: &'a str) {

        head {
            meta [ charset="utf-8" ] {}
            meta [ "http-equiv"="X-UA-Compatible", content="IE=edge"] {}
            meta [ name="viewport", content="width=device-width, initial-scale=1" ] {}
            title { {title} }

            script [ src = format!("/auth/static/{}", files::index_js.name),
                type="text/javascript", async=""] {}

            link [ rel = "stylesheet", type="text/css" , href = format!("/auth/static/{}", files::index_css.name)] {}
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
    AuthenticationLayout<'a>(content: &'a str, title: &'a str) {

        @markup::doctype()

        html[lang="en"] {

            @Header {
                title,
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

pub fn session_layout(title: &str, content: &str, use_hcaptcha: bool) -> HttpResponse {
    // https://docs.hcaptcha.com/#content-security-policy-settings
    let csp_content = if use_hcaptcha {
        format!(
            "default-src 'none'; script-src {}; frame-src {}; style-src {}; connect-src {};",
            "'self' 'unsafe-inline' https://hcaptcha.com https://*.hcaptcha.com",
            "https://hcaptcha.com https://*.hcaptcha.com",
            "'self' https://hcaptcha.com https://*.hcaptcha.com",
            "https://hcaptcha.com https://*.hcaptcha.com",
        )
    } else {
        "default-src 'none'; script-src 'self'; style-src 'self';".to_string()
    };

    let l = AuthenticationLayout { title, content };
    HttpResponse::Ok()
        .content_type("text/html")
        .append_header((
            http::header::CONTENT_SECURITY_POLICY,
            csp_content
        ))
        .append_header((
            http::header::X_FRAME_OPTIONS, "deny",
        ))
        .append_header((
            http::header::X_XSS_PROTECTION, "1; mode=block",
        ))
        .append_header((
            http::header::X_CONTENT_TYPE_OPTIONS, "nosniff",
        ))
        .append_header((
            http::header::REFERRER_POLICY, "no-referrer",
        ))
        .append_header((
            http::header::HeaderName::from_static("x-download-options"), "noopen",
        ))
        .append_header((
            http::header::X_DNS_PREFETCH_CONTROL, "off",
        ))
        .append_header((
            http::header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains",
        ))
        .append_header((
            http::header::HeaderName::from_static("permissions-policy"),
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
        ))
        .body(l.to_string())
}
