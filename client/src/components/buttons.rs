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
        _ => "bg-gradient-to-r from-amber-500 to-amber-600 text-gray-900"
    };
    
    let size_class = match props.size.as_str() {
        "sm" => "px-4 py-2 text-sm",
        "md" => "px-5 py-2.5 text-base",
        "lg" => "px-6 py-3 text-lg",
        _ => "px-5 py-2.5 text-base"
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



#[component]
pub fn Test() -> Element {
    rsx! {
        // Main container with Tailwind classes
        div {
            id:"test-container",
            class: "p-8 flex flex-col items-center gap-4 bg-gray-100 min-h-screen",
            // Test div 1 - Basic dimensions
            div {
                id:"test-1",
                class: "h-[40px] w-[460px] flex items-center justify-center bg-white rounded-md shadow",
                "Dimensions (h-[40px] w-[460px])"
            }
            
            // Test div 2 - Background color
            div {
                id:"test-2",
                class: "h-[40px] w-[460px] bg-blue-500 flex items-center justify-center text-white font-medium rounded-md",
                "Background (bg-blue-500)"
            }
            
            // Test div 3 - Border
            div {
                id:"test-3",
                class: "h-[40px] w-[460px] border-2 border-red-500 flex items-center justify-center rounded-md",
                "Border (border-2 border-red-500)"
            }
            
            // Test div 4 - Border radius
            div {
                id:"test-4",
                class: "h-[40px] w-[460px] bg-gray-200 rounded-xl flex items-center justify-center",
                "Border radius (rounded-xl)"
            }
            
            // Test div 5 - Shadow
            div {
                id:"test-5",
                class: "h-[40px] w-[460px] bg-white shadow-lg flex items-center justify-center rounded-md",
                "Shadow (shadow-lg)"
            }
            
            // Test div 6 - Hover and transition effects
            div {
                id:"test-6",
                class: "h-[40px] w-[460px] bg-green-500 hover:bg-green-600 transition-colors duration-300 flex items-center justify-center text-white rounded-md",
                "Hover effect (hover:bg-green-600)"
            }
            
            // Test div 7 - Gradient
            div {
                id:"test-7",
                class: "h-[40px] w-[460px] bg-gradient-to-r from-purple-500 to-pink-500 flex items-center justify-center text-white rounded-md",
                "Gradient (from-purple-500 to-pink-500)"
            }
        }
    }
}
