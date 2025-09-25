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
        // Explicitly set the document title
        document::Title { "FerrisUp - Rust Project Bootstrapping Tool" }
        
        // Meta tags for SEO
        document::Meta { charset: "utf-8" }
        document::Meta { name: "description", content: "FerrisUp - Powerful Rust project bootstrapping tool. Create, transform, and scale Rust projects with intelligent workspace management, component-based architecture, and framework-specific templates." }
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1.0" }
        document::Meta { name: "keywords", content: "rust, ferrisup, bootstrapping, cli, tool, web development, rust framework, project generator, rust templates, workspace management, rust components" }
        document::Meta { name: "author", content: "JITPOMI" }
        document::Meta { name: "robots", content: "index, follow" }
        document::Meta { name: "theme-color", content: "#fbbf24" }
        document::Meta { name: "color-scheme", content: "dark" }
        
        // Performance hints
        document::Meta { http_equiv: "x-dns-prefetch-control", content: "on" }
        document::Link { rel: "dns-prefetch", href: "//fonts.googleapis.com" }
        document::Link { rel: "dns-prefetch", href: "//github.com" }
        document::Link { rel: "dns-prefetch", href: "//crates.io" }
        
        // Open Graph meta tags
        document::Meta { property: "og:title", content: "FerrisUp" }
        document::Meta { property: "og:description", content: "Start Anywhere, Scale Anywhere with FerrisUp" }
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:url", content: "https://ferrisup.jitpomi.com/" }
        document::Meta { property: "og:image", content: "https://raw.githubusercontent.com/Jitpomi/ferrisup/main/img.png" }
        
        // Twitter Card meta tags
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        
        // Structured data for SEO
        script {
            r#type: "application/ld+json",
            {format!(r#"{{
                "@context": "https://schema.org",
                "@type": "SoftwareApplication",
                "name": "FerrisUp",
                "description": "Powerful Rust project bootstrapping tool for creating, transforming, and scaling Rust projects",
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
                    font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, 'Noto Sans', sans-serif;
                    margin: 0;
                    padding: 0;
                    line-height: 1.5;
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
        document::Link { rel: "preload", href: FERRISUP_LOGO_PNG, r#as: "image" }
        
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
