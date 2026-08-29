use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use std::string::ToString;

#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    #[props(default = "primary".to_string())]
    pub variant: String,
    #[props(default = "md".to_string())]
    pub size: String,
    pub children: Element,
    #[props(optional)]
    pub href: Option<String>,
    #[props(default = "_self".to_string())]
    pub target: String,
    #[props(optional)]
    pub rel: Option<String>,
    #[props(optional)]
    pub aria_label: Option<String>,
    // pub onclick: EventHandler<MouseData>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    // Determine button classes based on variant and size
    let variant_class = match props.variant.as_str() {
        "primary" => "bg-gradient-to-r from-amber-500 to-amber-600 text-gray-900",
        "secondary" => "bg-gray-700 text-white",
        "outline" => "bg-transparent border border-amber-500 text-amber-500",
        _ => "bg-gradient-to-r from-amber-500 to-amber-600 text-gray-900",
    };

    let size_class = match props.size.as_str() {
        "sm" => "px-4 py-2 text-sm",
        "md" => "px-5 py-2.5 text-base",
        "lg" => "px-6 py-3 text-lg",
        _ => "px-5 py-2.5 text-base",
    };

    let base_class = "relative inline-flex items-center justify-center font-semibold rounded-xl shadow-md transition-all duration-300 ease-in-out hover:scale-105 focus:outline-none active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed";

    // Render as anchor tag if href is provided, otherwise as button
    if let Some(href) = &props.href {
        let rel_value = props.rel.clone().unwrap_or_default();
        let aria_label = props.aria_label.clone();

        rsx! {
            a {
                href: "{href}",
                target: "{props.target}",
                rel: if !rel_value.is_empty() { "{rel_value}" } else { "" },
                aria_label: aria_label,
                class: "{base_class} {variant_class} {size_class}",
                span {
                    class: "absolute inset-0 rounded-xl bg-white/10 opacity-0 hover:opacity-100 transition-opacity duration-300",
                }
                span {
                    class: "relative z-10 flex items-center gap-2",
                    {props.children}
                }
            }
        }
    } else {
        let aria_label = props.aria_label.clone();

        rsx! {
            button {
                aria_label: aria_label,
                class: "{base_class} {variant_class} {size_class}",
                span {
                    class: "absolute inset-0 rounded-xl bg-white/10 opacity-0 hover:opacity-100 transition-opacity duration-300",
                }
                span {
                    class: "relative z-10 flex items-center gap-2",
                    {props.children}
                }
            }
        }
    }
}
