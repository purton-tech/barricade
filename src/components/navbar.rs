use std::vec::Vec;

markup::define! {
    MenuItem<'a>(name: &'a str, link: &'a str) {
        a [ href = link ] {
            {name}
        }
    }
    NavBar<'a>(title: &'a str, menu_items: &'a Vec<MenuItem<'a>>) {
        div["data-controller" = "navbar_type.get_controller()"] {
            div["data-target" = "navbar.navbar"] {
                nav {
                    input#toggle [ type="checkbox"] {}
                    label.hamburger [ for="toggle"] {
                        div.top_bun {}
                        div.meat {}
                        div.bottom_bun {}
                    }
                    a.logo [ href="/" ] {
                        span {
                            {title}
                        }
                    }
                    ul {

                        @for menu_item in menu_items.iter() {
                            li {
                                { menu_item }
                            }
                        }
                    }
                }
            }
        }
    }
}
