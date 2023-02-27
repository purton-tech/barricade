use crate::layout::Layout;
use dioxus::prelude::*;

pub struct Props {
    pub protected_symmetric_key: String,
    pub public_ecdh_key: String,
    pub protected_ecdh_private_key: String,
    pub public_ecdsa_key: String,
    pub protected_ecdsa_private_key: String,
    pub email: String
}

pub fn decrypt(props: Props) -> String {
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
                            "type": "hidden",
                            "data-target": "master.protectedSymmetricKey",
                            value: "{cx.props.protected_symmetric_key}"
                        }
                        input {
                            "type": "hidden",
                            "data-target": "master.publicECDHKey",
                            value: "{cx.props.public_ecdh_key}"
                        }
                        input {
                            "type": "hidden",
                            "data-target": "master.protectedECDHPrivateKey",
                            value: "{cx.props.protected_ecdh_private_key}"
                        }
                        input {
                            "type": "hidden",
                            "data-target": "master.publicECDSAKey",
                            value: "{cx.props.public_ecdsa_key}"
                        }
                        input {
                            "type": "hidden",
                            "data-target": "master.protectedECDSAPrivateKey",
                            value: "{cx.props.protected_ecdsa_private_key}"
                        }
                        input {
                            "type": "hidden",
                            "data-target": "master.email",
                            value: "{cx.props.email}"
                        }
                    }
                }
            }
        })
    }

    // Construct our component and render it to a string.
    let mut app = VirtualDom::new_with_props(
        app,
        props,
    );
    let _ = app.rebuild();
    dioxus::ssr::render_vdom(&app)
}