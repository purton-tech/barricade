use crate::layout::Layout;
use db::User;
use dioxus::prelude::*;
use super::SIGN_IN;

struct Props {
    _users: Vec<User>
}

pub fn sign_in(users: Vec<User>) -> String {

    // Inner function to create our rsx! component
    fn app(cx: Scope<Props>) -> Element {
        cx.render(rsx! {
            Layout {
                title: "Logon",
                form {
                    method: "post",
                    action: "{SIGN_IN}",
                    label {
                        "for": "email",
                        "Email Address"
                    }
                    input {
                        "type": "email",
                        required: "required",
                        placeholder: "Email Address"
                    }
                    button {
                        "type": "submit",
                        "Continue with email"
                    }
                }
            }
        })
    }

    // Construct our component and render it to a string.
    let mut app = VirtualDom::new_with_props(
        app,
        Props {
            _users: users
        },
    );
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}