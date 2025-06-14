//! Library template created with FerrisUp

use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use std::string::ToString;

#[derive(PartialEq, Clone)]
pub struct LinkType {
    pub id: String,
    pub href: String,
    pub icon: String,
    pub label: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct HeroProps {
    pub src: String,
    pub links: Vec<LinkType>,
}

// Sample links (should be in main or passed as props — can't be `const`)

#[component]
pub fn Hero(props: HeroProps) -> Element {
    rsx! {
        // Main container with test Tailwind classes
        div {
            id: "hero",
            class: "flex flex-col items-center justify-center min-h-screen p-8",
            style: "background-color: black;",
            // Image with Tailwind classes
            img {
                src: "{props.src}",
                id: "header",
                class: "h-auto w-[300px] max-w-md rounded-lg shadow-2xl mb-12 hover:scale-105 transition-transform duration-300",
                alt: "FerrisUp Logo"
            }
            
            // Links container with Tailwind classes
            div {
                id: "links",
                class: "flex bg-red-950 flex-wrap justify-center gap-4 w-full max-w-2xl",
                
                // Map through links with Tailwind classes
                {
                    props.links.iter().map(|link| rsx! {
                        a {
                            id: "{link.id}",
                            href: "{link.href}",
                            class: "w-[200px] border border-white/20 shadow-lg",
                            target: "_blank",
                            rel: "noopener noreferrer",
                            
                            // Icon
                            span {
                                class: "text-xl",
                                "{link.icon}"
                            }
                            
                            // Label
                            span {
                                class: "font-medium",
                                "{link.label}"
                            }
                        }
                    })
                }
            }

        }
    }
}

/// Returns a greeting message
pub fn hello() -> String {
    "Hello from FerrisUp library template!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from FerrisUp library template!");
    }
}
