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
                title: "Encryption Password",
                div {
                    "data-controller": "registration password",
                    form {
                        class: "m_authentication",
                        h1 {
                            "Create an encryption password"
                        }
                        label {
                            "for": "password",
                            "Please enter a password"
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

                        span {
                            class: "a_help_text",
                            "data-target": "password.help",
                            "The master password is the password we use to generate your private keys. 
                            It is very important that you do not forget your master password. 
                            There is no way to recover the password in the event that you forget it."
                        }
                        
                        span {
                            class: "a_help_text",
                            "data-target": "password.warning"
                        }
                        
                        span {
                            class: "a_help_text",
                            "data-target": "password.suggestions"
                        }
                        label {
                            "for": "confirm_password",
                            "Re-type Master Password"
                        }
                        input {
                            id: "confirm_password",
                            name: "confirm_password",
                            autocomplete: "new-password",
                            "type": "password",
                            required: "required",
                            "data-target": "registration.confirmPassword"
                        }
                        // This actually gets used by password managers.
                        // https://stackoverflow.com/questions/48525114/chrome-warning-dom-password-forms-should-have-optionally-hidden-username-fi
                        input {
                            id: "email",
                            name: "email",
                            value: "{cx.props.email}",
                            hidden: "hidden",
                            autocomplete: "username",
                            "type": "text",
                            "data-target": "registration.email"
                        }

                        button {
                            "data-target":  "registration.button password.button",
                            "data-action": "registration#register",
                            class: "a_button success",
                            "type": "submit",
                            "Submit"
                        }
                    }

                    // This is the form that actually gets sent to the server. It's populated
                    // by the javascript.
                    form {
                        method: "post",
                        "data-target": "registration.form",
                        input {
                            name: "master_password_hash",
                            "data-target": "registration.masterPasswordHash",
                            "type": "hidden"
                        },
                        input {
                            name: "protected_symmetric_key",
                            "data-target": "registration.protectedSymmetricKey",
                            "type": "hidden"
                        },
                        input {
                            name: "protected_ecdh_private_key",
                            "data-target": "registration.protectedECDHPrivateKey",
                            "type": "hidden"
                        },
                        input {
                            name: "ecdh_public_key",
                            "data-target": "registration.publicECDHKey",
                            "type": "hidden"
                        },
                        input {
                            name: "protected_ecdsa_private_key",
                            "data-target": "registration.protectedECDSAPrivateKey",
                            "type": "hidden"
                        },
                        input {
                            name: "ecdsa_public_key",
                            "data-target": "registration.publicECDSAKey",
                            "type": "hidden"
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