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

const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // Explicitly set the document title
        document::Title { "FerrisUp - Rust Project Bootstrapping Tool" }

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

        // Twitter Card meta tags
        document::Meta { name: "twitter:card", content: "summary_large_image" }

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

        // Critical inline CSS for FCP optimization
        style {
            {format!(r#"
                body {{
                    background-color: #101828;
                    color: #ffffff;
                    font-family: 'Source Sans 3', 'Segoe UI', sans-serif;
                    margin: 0;
                    padding: 0;
                    line-height: 1.5;
                }}
                h1, h2, h3, h4, h5, h6 {{
                    font-family: 'Space Grotesk', 'Segoe UI', sans-serif;
                    letter-spacing: -0.025em;
                }}
                button, a {{
                    font-family: 'Source Sans 3', 'Segoe UI', sans-serif;
                }}
                code, pre {{
                    font-family: 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace;
                    font-variant-ligatures: none;
                }}
                .hero-container {{
                    background-color: #111827;
                    min-height: 100vh;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                    text-align: center;
                    padding: 2rem 1rem;
                }}
                .hero-title {{
                    font-size: 3rem;
                    font-weight: 700;
                    color: white;
                    margin-bottom: 1rem;
                    line-height: 1.2;
                }}
                .hero-subtitle {{
                    font-size: 1.5rem;
                    color: #fbbf24;
                    margin-bottom: 1.5rem;
                }}
                .hero-logo {{
                    width: 240px;
                    height: 240px;
                    border-radius: 50%;
                    margin-bottom: 2rem;
                }}
                @media (max-width: 768px) {{
                    .hero-title {{ font-size: 2rem; }}
                    .hero-subtitle {{ font-size: 1.25rem; }}
                    .hero-logo {{ width: 180px; height: 180px; }}
                }}
            "#)}
        }

        // Preload critical assets

        // Defer non-critical CSS
        document::Link { rel: "preload", href: TAILWIND_CSS, r#as: "style", onload: "this.onload=null;this.rel='stylesheet'" }
        noscript {
            document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        }
        document::Link { rel: "stylesheet", href: MAIN_CSS, media: "print", onload: "this.media='all'" }

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
