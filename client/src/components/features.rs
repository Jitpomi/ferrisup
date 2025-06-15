use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use crate::components::feature_card::FeatureCard;

#[component]
pub fn Features() -> Element {
    rsx! {
        section {
            class: "py-24 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto relative",
            
            // Background elements for premium feel
            div {
                class: "absolute inset-0 bg-gradient-to-b from-[#121212] via-[#151515] to-[#181818] -z-10"
            }
            
            // Decorative elements
            div {
                class: "absolute top-0 left-0 w-full h-px bg-gradient-to-r from-transparent via-amber-500/30 to-transparent"
            }
            
            // Section header with modern styling
            div {
                class: "text-center mb-16",
                span {
                    class: "inline-block px-3 py-1 text-sm font-medium bg-amber-900/30 text-amber-400 rounded-full mb-4",
                    "Features"
                }
                h2 {
                    class: "text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-amber-400 to-orange-500 mb-4",
                    "FerrisUp Features"
                }
                p {
                    class: "text-gray-400 max-w-2xl mx-auto text-lg",
                    "A versatile Rust project bootstrapping tool that enables developers to create, transform, and scale Rust projects with ease."
                }
            }
            
            // Feature grid with staggered animation effect
            div {
                class: "grid grid-cols-1 md:grid-cols-3 gap-8 lg:gap-10",
                
                // Feature 1
                FeatureCard {
                    icon: "🔄".to_string(),
                    title: "Project Transformation".to_string(),
                    description: "Convert single-crate projects to workspaces as they grow. Start simple and evolve your project structure without starting from scratch.".to_string()
                }
                
                // Feature 2
                FeatureCard {
                    icon: "🧩".to_string(),
                    title: "Component-Based Architecture".to_string(),
                    description: "Specialized components for different use cases. Mix and match components to build the perfect project structure for your needs.".to_string()
                }
                
                // Feature 3
                FeatureCard {
                    icon: "📋".to_string(),
                    title: "Domain-Specific Templates".to_string(),
                    description: "Optimized templates for web, data science, embedded, and more. Each template is designed for its specific domain with appropriate dependencies.".to_string()
                }
            }
            
            // Additional features section
            div {
                class: "mt-16 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6",
                
                // Mini feature 1
                div {
                    class: "bg-gray-800/50 p-6 rounded-xl border border-gray-700 hover:border-amber-500/50 transition-colors duration-300",
                    div {
                        class: "flex items-center mb-4",
                        span {
                            class: "text-amber-400 mr-3",
                            "🔧"
                        }
                        h3 {
                            class: "font-semibold text-white",
                            "Smart Dependency Management"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm",
                        "Interactive dependency handling with feature selection."
                    }
                }
                
                // Mini feature 2
                div {
                    class: "bg-gray-800/50 p-6 rounded-xl border border-gray-700 hover:border-amber-500/50 transition-colors duration-300",
                    div {
                        class: "flex items-center mb-4",
                        span {
                            class: "text-amber-400 mr-3",
                            "📦"
                        }
                        h3 {
                            class: "font-semibold text-white",
                            "Workspace Support"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm",
                        "Transform single-crate projects into workspaces as they grow."
                    }
                }
                
                // Mini feature 3
                div {
                    class: "bg-gray-800/50 p-6 rounded-xl border border-gray-700 hover:border-amber-500/50 transition-colors duration-300",
                    div {
                        class: "flex items-center mb-4",
                        span {
                            class: "text-amber-400 mr-3",
                            "🌐"
                        }
                        h3 {
                            class: "font-semibold text-white",
                            "Multiple Framework Support"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm",
                        "Support for Axum, Leptos, Dioxus, Actix, and more."
                    }
                }
                
                // Mini feature 4
                div {
                    class: "bg-gray-800/50 p-6 rounded-xl border border-gray-700 hover:border-amber-500/50 transition-colors duration-300",
                    div {
                        class: "flex items-center mb-4",
                        span {
                            class: "text-amber-400 mr-3",
                            "🚀"
                        }
                        h3 {
                            class: "font-semibold text-white",
                            "Extensible Templates"
                        }
                    }
                    p {
                        class: "text-gray-400 text-sm",
                        "Create and share your own project templates and transformations."
                    }
                }
            }
        }
    }
}
