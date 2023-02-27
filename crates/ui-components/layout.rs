#![allow(non_snake_case)]

use dioxus::prelude::*;
use assets::files::*;

// Remember: owned props must implement PartialEq!
#[derive(Props)]
pub struct AppLayoutProps<'a> {
    title: &'a str,
    children: Element<'a>,
}

pub fn Layout<'a>(cx: Scope<'a, AppLayoutProps<'a>>) -> Element {
    cx.render(rsx!(
        {
            LazyNodes::new(|f| f.text(format_args!("<!DOCTYPE html><html lang='en'>")))
        }
        head {
            title {
                "{cx.props.title}"
            }
            meta {
                charset: "utf-8"
            }
            meta {
                "http-equiv": "X-UA-Compatible",
                content: "IE=edge"
            }
            meta {
                name: "viewport",
                content: "width=device-width, initial-scale=1"
            }
            link {
                rel: "stylesheet",
                href: "/auth{index_css.name}",
                "type": "text/css"
            }
            script {
                "type": "module",
                src: "/auth{index_js.name}"
            }
        }
        body {
            &cx.props.children
        }
    ))
}