use dioxus::prelude::*;
use crate::components::buttons::Button;
use crate::HEADER_SVG;

#[component]
pub fn Hero() -> Element {
    
    rsx! {
        // Using article for better semantic structure
        article {
            class: "relative py-20 bg-gray-900 overflow-hidden font-['Inter',system-ui,sans-serif]",
            // Adding role for accessibility and SEO
            role: "banner",
            aria_labelledby: "hero-heading",
            
            // Container for content centering and max-width
            div {
                class: "container mx-auto px-4 relative z-10",
            }
            
            // Hero content with flex layout for better alignment
            main {
                class: "flex flex-col items-center text-center",
                
                // Logo area with FerrisUp image - improved alt text
                figure {
                    class: "flex items-center justify-center mb-4",
                    img {
                        class: "w-60 h-60 rounded-full shadow-lg",
                        src: HEADER_SVG,
                        alt: "FerrisUp - Rust Project Bootstrapping Tool Logo",
                        loading: "eager",
                        width: "240",
                        height: "240"
                    }
                }
                
                h1 {
                    class: "text-4xl md:text-5xl lg:text-6xl font-bold text-white font-montserrat leading-tight",
                    id: "hero-heading",
                    "Rust Project Bootstrapping Tool"
                }
                
                // Tagline with improved semantic structure
                h2 {
                    class: "mt-4 text-xl md:text-2xl text-amber-300 font-medium font-roboto tracking-wide",
                    "Start Anywhere, Scale Anywhere with Rust"
                }
                
                // Description with better keywords
                p {
                    class: "mt-6 text-lg text-gray-300 max-w-3xl mx-auto font-opensans",
                    "FerrisUp is a powerful Rust project bootstrapping tool designed for modern Rust developers. Create, transform, and scale Rust projects with intelligent workspace management, component-based architecture, and framework-specific templates. Seamlessly convert single-crate projects to workspaces as they grow, with specialized components for web, data science, embedded systems, and serverless applications."
                }
                
                // Version badge - adds a premium touch
                div {
                    class: "mt-4 mb-8 inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-amber-900/30 text-amber-400 border border-amber-700/50",
                    "v0.1.23"
                }
                
                // Call to action buttons with improved styling and accessibility
                nav {
                    class: "mt-8 flex flex-col sm:flex-row gap-5 justify-center",
                    aria_label: "Primary navigation",
                    Button {
                        variant: "primary",
                        href: "https://crates.io/crates/ferrisup",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        aria_label: "View FerrisUp on Crates.io",
                        "View on Crates.io"
                    }
                    
                    a {
                        href: "https://github.com/Jitpomi/ferrisup",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        aria_label: "Read FerrisUp Documentation",
                        class: "group relative inline-flex items-center justify-center px-6 py-3 text-base font-medium rounded-xl bg-gray-800 text-amber-400 border border-amber-700/50 transition-all duration-300 hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-amber-500",
                        span {
                            class: "flex items-center gap-2",
                            "Documentation"
                            svg {
                                class: "w-4 h-4 ml-1 transition-transform duration-300 group-hover:translate-x-1",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M9 5l7 7-7 7"
                                }
                            }
                        }
                    }
                }
                
                // Terminal-like display showing example code with better semantic structure
                section {
                    class: "mt-12 w-full max-w-3xl mx-auto overflow-hidden rounded-lg shadow-lg",
                    aria_labelledby: "terminal-heading",
                    div {
                        class: "flex items-center bg-gray-900 px-4 py-2 border-b border-gray-800",
                        div {
                            class: "flex space-x-2",
                            span { class: "w-3 h-3 rounded-full bg-red-500" }
                            span { class: "w-3 h-3 rounded-full bg-yellow-500" }
                            span { class: "w-3 h-3 rounded-full bg-green-500" }
                        }
                        div {
                            class: "flex-grow text-center text-sm text-gray-400",
                            id: "terminal-heading",
                            "FerrisUp CLI Examples"
                        }
                    }
                    pre {
                        class: "bg-[#0d1117] p-6 overflow-x-auto text-left",
                        code {
                            class: "text-[10px] xs:text-[11px] sm:text-xs md:text-sm font-mono leading-relaxed",
                            
                            // Installation
                            div {
                                class: "block text-gray-500",
                                "# Installation"
                            }
                            
                            div {
                                class: "block",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "cargo install ferrisup" }
                            }
                            
                            // Create new projects
                            div {
                                class: "block mt-4 text-gray-500",
                                "# Create new projects"
                            }
                            
                            div {
                                class: "block",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "ferrisup new my_project" }
                            }
                            
                            div {
                                class: "block mt-2",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "ferrisup new my_client --component-type client --framework leptos" }
                            }
                            
                            div {
                                class: "block mt-2",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "ferrisup new my_server --component-type server --framework axum" }
                            }
                            
                            // Transform projects
                            div {
                                class: "block mt-4 text-gray-500",
                                "# Transform projects"
                            }
                            
                            div {
                                class: "block",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "ferrisup transform -p my_server" }
                            }
                            
                            // List available components
                            div {
                                class: "block mt-4 text-gray-500",
                                "# List available components"
                            }
                            
                            div {
                                class: "block",
                                span { class: "text-green-400", "$ " }
                                span { class: "text-yellow-400", "ferrisup list" }
                            }
                        }
                    }
                }
                
                // Adding structured data for SEO (hidden visually but available for search engines)
                div {
                    class: "hidden",
                    itemscope: true,
                    itemtype: "https://schema.org/SoftwareApplication",
                    meta { itemprop: "name", content: "FerrisUp" }
                    meta { itemprop: "description", content: "Rust project bootstrapping tool for modern Rust developers" }
                    meta { itemprop: "applicationCategory", content: "DeveloperApplication" }
                    meta { itemprop: "operatingSystem", content: "Cross-platform" }
                    meta { itemprop: "offers", itemscope: true, itemtype: "https://schema.org/Offer",
                        meta { itemprop: "price", content: "0" }
                        meta { itemprop: "priceCurrency", content: "USD" }
                    }
                }
            }
        }
    }
}
