use assets::files;
use axum::{http::{header, HeaderMap}, response::IntoResponse};

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

pub fn session_layout(title: &str, content: &str, use_hcaptcha: bool) -> impl IntoResponse {
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
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/html; charset=utf-8".parse().unwrap());
    headers.insert(header::CONTENT_SECURITY_POLICY, csp_content.parse().unwrap());
    headers.insert(header::X_FRAME_OPTIONS, "deny".parse().unwrap());
    headers.insert(header::X_XSS_PROTECTION, "1; mode=block".parse().unwrap());
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    //headers.insert(header::X_DOWNLOAD_OPTIONS, "noopen".parse().unwrap());
    headers.insert(header::X_DNS_PREFETCH_CONTROL, "off".parse().unwrap());
    headers.insert(header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains".parse().unwrap());
    //headers.insert(header::PERMISSIONS_POLICY, "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()".parse().unwrap());
    (headers, l.to_string())
}
