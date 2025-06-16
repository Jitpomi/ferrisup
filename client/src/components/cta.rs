use dioxus::prelude::*;
use crate::components::buttons::Button;

// Background component with gradient and pattern overlay
#[component]
fn CtaBackground() -> Element {
    rsx! {
        div {
            class: "absolute inset-0 -z-10",
            // Gradient overlay
            div {
                class: "absolute inset-0 bg-gradient-to-br from-amber-900/80 to-gray-900 mix-blend-multiply"
            }
            // Pattern overlay
            div {
                class: "absolute inset-0 opacity-10 bg-pattern"
            }
        }
    }
}

// GitHub button component
#[component]
fn GitHubButton() -> Element {
    rsx! {
        a {
            href: "https://github.com/Jitpomi/ferrisup",
            target: "_blank",
            rel: "noopener noreferrer",
            aria_label: "View FerrisUp source code on GitHub",
            class: "inline-flex items-center justify-center px-6 py-3 text-base font-medium rounded-xl bg-amber-900/30 text-amber-300 border border-amber-700/50 hover:bg-amber-900/50 transition-colors duration-300",
            
            // GitHub icon
            svg {
                class: "w-5 h-5 mr-2",
                xmlns: "http://www.w3.org/2000/svg",
                view_box: "0 0 24 24",
                fill: "currentColor",
                path {
                    d: "M12 0C5.37 0 0 5.37 0 12c0 5.3 3.44 9.8 8.2 11.4.6.1.8-.3.8-.6v-2.2c-3.3.7-4-1.4-4-1.4-.5-1.4-1.3-1.8-1.3-1.8-1.1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1.1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.8-1.6-2.7-.3-5.5-1.3-5.5-5.9 0-1.3.5-2.4 1.2-3.2-.1-.3-.5-1.5.1-3.2 0 0 1-.3 3.3 1.2 1-.3 2-.4 3-.4s2 .1 3 .4c2.3-1.6 3.3-1.2 3.3-1.2.7 1.7.2 2.9.1 3.2.8.8 1.2 1.9 1.2 3.2 0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6 4.8-1.6 8.2-6.1 8.2-11.4C24 5.37 18.63 0 12 0z"
                }
            }
            "View on GitHub"
        }
    }
}

// Terminal header component
#[component]
fn TerminalHeader() -> Element {
    rsx! {
        header {
            class: "flex items-center bg-gray-800 px-4 py-2 border-b border-gray-700",
            div {
                class: "flex space-x-2",
                span { class: "w-3 h-3 rounded-full bg-red-500" }
                span { class: "w-3 h-3 rounded-full bg-yellow-500" }
                span { class: "w-3 h-3 rounded-full bg-green-500" }
            }
            div {
                class: "ml-4 text-sm text-gray-400",
                "example.rs"
            }
        }
    }
}

// Simple code snippet component
#[component]
fn SimpleCodeLine( class: &'static str, content: &'static str) -> Element {
  rsx! {
        div {
            class: "{class}",
            "{content}"
        }
    }
}


// Terminal component that combines header and code snippet
#[component]
fn CodeTerminal() -> Element {
    rsx! {
        figure {
            class: "bg-gray-900 rounded-xl overflow-hidden shadow-2xl border border-amber-500/20",
            aria_label: "FerrisUp code examples",
            TerminalHeader {}
            CodeSnippet {}
        }
    }
}

// Content text component
#[component]
fn CtaContent() -> Element {
    rsx! {
        article {
            class: "lg:w-3/5 text-center lg:text-left",
            // Badge
            div {
                class: "inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-amber-900/30 text-amber-300 border border-amber-700/50 mb-4",
                "Key Features"
            }
            // Heading
            h2 {
                class: "text-3xl sm:text-4xl font-bold text-white mb-4",
                id: "features-heading",
                "What Makes FerrisUp Different?"
            }
            // Description
            p {
                class: "text-lg text-gray-300 mb-6 max-w-2xl mx-auto lg:mx-0",
                "Unlike traditional template generators like cargo-generate, FerrisUp focuses on project evolution. Start with a simple project and transform it as your needs grow, without having to recreate your project structure from scratch."
            }
            
            // Key features list
            ul {
                class: "list-disc list-inside text-gray-300 mb-8 max-w-2xl mx-auto lg:mx-0 space-y-2",
                aria_labelledby: "features-heading",
                li { "Project Transformation - Convert single-crate projects to workspaces as they grow" }
                li { "Component-Based Architecture - Specialized components for different use cases" }
                li { "Domain-Specific Templates - Optimized templates for web, data science, embedded, and more" }
                li { "Smart Dependency Management - Interactive dependency handling with feature selection" }
            }
            // Buttons
            nav {
                class: "flex flex-col sm:flex-row gap-4 justify-center lg:justify-start",
                aria_label: "Download and source code links",
                Button {
                    variant: "primary",
                    size: "lg",
                    href: "https://crates.io/crates/ferrisup",
                    target: "_blank",
                    rel: "noopener noreferrer",
                    aria_label: "Download FerrisUp from Crates.io",
                    "View on Crates.io"
                }
                GitHubButton {}
            }
        }
    }
}

// Main CallToAction component that composes all the smaller components
#[component]
pub fn CallToAction() -> Element {
    rsx! {
        section {
            class: "py-20 px-4 sm:px-6 lg:px-8 relative overflow-hidden",
            id: "features",
            aria_labelledby: "features-heading",
            CtaBackground {}
            div {
                class: "max-w-5xl mx-auto relative z-10",
                div {
                    class: "flex flex-col lg:flex-row items-center justify-between gap-12",
                    CtaContent {}
                    div {
                        class: "lg:w-2/5 w-full",
                        CodeTerminal {}
                    }
                }
            }
            
            // Hidden structured data for SEO
            div {
                class: "hidden",
                itemscope: true,
                itemtype: "https://schema.org/SoftwareApplication",
                meta { itemprop: "name", content: "FerrisUp" }
                meta { itemprop: "description", content: "Rust project bootstrapping tool with project transformation capabilities" }
                meta { itemprop: "applicationCategory", content: "DeveloperApplication" }
                meta { itemprop: "keywords", content: "Rust, project bootstrapping, workspace management, component architecture" }
                div {
                    itemprop: "featureList",
                    itemscope: true,
                    itemtype: "https://schema.org/ItemList",
                    meta { itemprop: "itemListElement", content: "Project Transformation" }
                    meta { itemprop: "itemListElement", content: "Component-Based Architecture" }
                    meta { itemprop: "itemListElement", content: "Domain-Specific Templates" }
                    meta { itemprop: "itemListElement", content: "Smart Dependency Management" }
                }
            }
        }
    }
}


// Code snippet component with simplified structure
#[component]
fn CodeSnippet() -> Element {
    rsx! {
        div {
            class: "p-4 bg-gray-950 text-gray-300 font-mono text-sm overflow-x-auto",
            role: "region",
            aria_label: "FerrisUp command examples",
            // Quick Start examples from README
            div {
                class: "mb-2 text-amber-300 font-semibold",
                "# Quick Start Examples"
            }
            div {
                class: "mb-2",
                span { class: "text-green-400", "$ " }
                span { class: "text-amber-400", "ferrisup new my_project" }
            }
            div {
                class: "mb-2",
                span { class: "text-green-400", "$ " }
                span { class: "text-amber-400", "ferrisup new my_client --component-type client --framework leptos" }
            }
            div {
                class: "mb-2",
                span { class: "text-green-400", "$ " }
                span { class: "text-amber-400", "ferrisup new my_server --component-type server --framework axum" }
            }
            div {
                class: "mb-2",
                span { class: "text-green-400", "$ " }
                span { class: "text-amber-400", "ferrisup transform -p my_server" }
                span { class: "text-gray-500", " # Convert to workspace" }
            }
        }
    }
}
