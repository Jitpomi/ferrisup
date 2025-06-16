// Client application for FerrisUp
use dioxus::prelude::*;

mod components;
use components::home::HomePage;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("assets/favicon.ico");
const HEADER_SVG: Asset = asset!("assets/img.png");

const FERRISUP_LOGO_PNG: Asset = asset!("assets/ferrisup-logo.png");
const FERRISUP_PNG: Asset = asset!("assets/ferrisup.png");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // These will be handled by Dioxus.toml configuration
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }
        Router::<Route> {}
    }
}


/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        HomePage {}
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    let blog_title = format!("This is blog {}!", id);
    let blog_desc = format!("In blog {}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components.", id);
    
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { {blog_title} }
            p { {blog_desc} }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            // Link {
            //     to: Route::Home {},
            //     "Home"
            // }
            // Link {
            //     to: Route::Blog { id: 1 },
            //     "Blog"
            // }
        }

        Outlet::<Route> {}
    }
}
