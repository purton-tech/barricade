use crate::layout::Layout;
use dioxus::prelude::*;
use super::SIGN_IN;

pub fn sign_in() -> String {

    // Inner function to create our rsx! component
    fn app(cx: Scope) -> Element {
        cx.render(rsx! {
            Layout {
                title: "Logon",
                form {
                    class: "m_authentication",
                    method: "post",
                    action: "{SIGN_IN}",
                    label {
                        "for": "email",
                        "Email Address"
                    }
                    input {
                        "type": "email",
                        name: "email",
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
    let mut app = VirtualDom::new(
        app
    );
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}