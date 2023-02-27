use crate::layout::Layout;
use dioxus::prelude::*;

struct Props {
    email: String
}

pub fn encryption_password(email: String) -> String {
    // Inner function to create our rsx! component
    fn app(cx: Scope<Props>) -> Element {
        cx.render(rsx! {
            Layout {
                title: "Decrypt Keys",
                div {
                    class: "m_decryption",
                    "data-controller": "master",
                    form {
                        "data-target": "master.form",
                        method: "post",

                        div {
                            class: "center",
                        }
                        input {
                            id: "password",
                            name: "password",
                            autocomplete: "new-password",
                            "type": "password",
                            required: "required",
                            "data-action": "keyup->password#keyPress",
                            "data-target": "registration.password password.password"
                        }
                    }
                }
            }
        })
    }

    // Construct our component and render it to a string.
    let mut app = VirtualDom::new_with_props(
        app,
        Props {
            email
        },
    );
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}