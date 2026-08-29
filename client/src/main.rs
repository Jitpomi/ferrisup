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
const FERRISUP_LOGO: Asset = asset!("assets/ferrisup-logo.png");
const FERRISUP_MARK: Asset = asset!("assets/ferrisup-mark.png");

// Bundle, preload, and apply CSS from the initial HTML before WebAssembly starts.
const _: Asset = asset!(
    "assets/tailwind.css",
    AssetOptions::css()
        .with_preload(true)
        .with_static_head(true)
);
const _: Asset = asset!(
    "assets/main.css",
    AssetOptions::css()
        .with_preload(true)
        .with_static_head(true)
);

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Explicitly set the document title
        document::Title { "FerrisUp — Rust Project Foundations" }

        // Meta tags for SEO
        document::Meta { charset: "utf-8" }
        document::Meta { name: "description", content: "FerrisUp creates inspectable Rust project foundations and helps single crates grow into clear Cargo workspaces for human and AI-assisted development." }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Meta { name: "keywords", content: "rust, ferrisup, cli, project generator, cargo workspace, rust templates, AI-assisted development" }
        document::Meta { name: "author", content: "JITPOMI" }
        document::Meta { name: "robots", content: "index, follow" }
        document::Meta { name: "theme-color", content: "#fbbf24" }
        document::Meta { name: "color-scheme", content: "dark" }

        // Performance hints
        document::Meta { http_equiv: "x-dns-prefetch-control", content: "on" }
        document::Link { rel: "dns-prefetch", href: "//fonts.googleapis.com" }
        document::Link { rel: "dns-prefetch", href: "//github.com" }
        document::Link { rel: "dns-prefetch", href: "//crates.io" }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com" }
        document::Link { rel: "stylesheet", href: "https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500&family=Source+Sans+3:opsz,wght@8..60,400;8..60,500;8..60,600&family=Space+Grotesk:wght@500;600;700&display=swap" }

        // Open Graph meta tags
        document::Meta { property: "og:title", content: "FerrisUp" }
        document::Meta { property: "og:description", content: "Start Anywhere, Scale Anywhere with FerrisUp" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:url", content: "https://ferrisup.jitpomi.com/" }
        document::Meta { property: "og:image", content: format!("https://ferrisup.jitpomi.com{}", FERRISUP_LOGO) }
        document::Meta { property: "og:image:alt", content: "FerrisUp crab logo and wordmark" }

        // Twitter Card meta tags
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:image", content: format!("https://ferrisup.jitpomi.com{}", FERRISUP_LOGO) }

        // Structured data for SEO
        script {
            r#type: "application/ld+json",
            {format!(r#"{{
                "@context": "https://schema.org",
                "@type": "SoftwareApplication",
                "name": "FerrisUp",
                "description": "Rust project foundations and Cargo workspace evolution for human and AI-assisted development",
                "url": "https://ferrisup.jitpomi.com",
                "applicationCategory": "DeveloperApplication",
                "operatingSystem": "Cross-platform",
                "programmingLanguage": "Rust",
                "author": {{
                    "@type": "Organization",
                    "name": "JITPOMI",
                    "url": "https://jitpomi.com"
                }},
                "downloadUrl": "https://crates.io/crates/ferrisup",
                "codeRepository": "https://github.com/Jitpomi/ferrisup",
                "license": "MIT",
                "keywords": ["rust", "cli", "bootstrapping", "project-generator", "templates"]
            }}"#)}
        }

        // Favicon and critical CSS
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "apple-touch-icon", href: FERRISUP_MARK }

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
    let blog_desc = format!(
        "In blog {}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components.",
        id
    );

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
