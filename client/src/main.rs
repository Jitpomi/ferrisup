// Client application for FerrisUp
use dioxus::prelude::*;

// Import our local components
mod components;
use components::*;
use crate::components::LinkType;

fn get_links() -> Vec<LinkType> {
    vec![
        LinkType {
            id: "docs".to_string(),
            href: "https://dioxuslabs.com/learn/0.6/".to_string(),
            icon: "📚".to_string(),
            label: "Docs".to_string(),
        },
        LinkType {
            id: "playground".to_string(),
            href: "https://dioxuslabs.com/awesome".to_string(),
            icon: "🚀".to_string(),
            label: "Playground".to_string(),
        },
        LinkType {
            id: "crateIO".to_string(),
            href: "https://dioxuslabs.com/awesome".to_string(),
            icon: "🎁".to_string(),
            label: "CrateIO".to_string(),
        },
        LinkType {
            id: "contribute".to_string(),
            href: "https://dioxuslabs.com/awesome".to_string(),
            icon: "🧩".to_string(),
            label: "Contribute".to_string(),
        },
        LinkType {
            id: "discord".to_string(),
            href: "https://dioxuslabs.com/awesome".to_string(),
            icon: "👋".to_string(),
            label: "Discord".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const HEADER_SVG: Asset = asset!("/assets/img.png");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet{ href: TAILWIND_CSS }
        Router::<Route> {}
    }
}


/// Home page
#[component]
fn Home() -> Element {
    rsx! {

        Hero {
            src: HEADER_SVG,
            links: get_links()
        }

    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

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
