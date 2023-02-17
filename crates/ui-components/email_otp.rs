use crate::layout::Layout;
use dioxus::prelude::*;

pub fn email_otp() -> String {
    // Inner function to create our rsx! component
    fn app(cx: Scope) -> Element {
        cx.render(rsx! {
            Layout {
                title: "Email OTP",
                form {
                    method: "post",
                    label {
                        "for": "otp",
                        "6 Digit Code"
                    }
                    input {
                        "type": "number",
                        required: "required",
                        placeholder: "6 Digit Code"
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
