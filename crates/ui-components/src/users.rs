use crate::layout::Layout;
use db::User;
use dioxus::prelude::*;

struct Props {
    users: Vec<User>
}

pub fn users(users: Vec<User>) -> String {
    fn app(cx: Scope<ProfileProps>) -> Element {
        cx.render(rsx! {
        })
    }

    let mut app = VirtualDom::new_with_props(
        app,
        Props {
            users
        },
    );
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}