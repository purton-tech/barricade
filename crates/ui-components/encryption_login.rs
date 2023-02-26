use crate::layout::Layout;
use dioxus::prelude::*;

pub fn encryption_password() -> String {
    // Inner function to create our rsx! component
    fn app(cx: Scope) -> Element {
        cx.render(rsx! {
            Layout {
                title: "Encryption Password",
                form {
                    class: "m_authentication",
                    method: "post",
                    label {
                        "for": "password",
                        "Please enter a password"
                    }
                    input {
                        "name": "password",
                        "type": "password",
                        required: "required"
                    }
                    button {
                        "type": "submit",
                        "Submit"
                    }
                }
            }
        })
    }

    // Construct our component and render it to a string.
    let mut app = VirtualDom::new(app);
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}