use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct FeatureCardProps {
    pub icon: String,
    pub title: String,
    pub description: String,
}

#[component]
pub fn FeatureCard(props: FeatureCardProps) -> Element {
    let mut hovered = use_signal(|| false);
    
    rsx! {
        div {
            class: "group relative p-8 rounded-2xl bg-gradient-to-b from-gray-800 to-gray-900 border border-gray-700 shadow-xl hover:shadow-2xl transition-all duration-500 overflow-hidden",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            
            // Glowing effect on hover
            div {
                class: "absolute inset-0 bg-gradient-to-r from-amber-500/10 to-orange-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-500"
            }
            
            // Animated corner accent
            div {
                class: "absolute -top-1 -right-1 w-12 h-12 bg-gradient-to-br from-amber-400 to-orange-600 rounded-bl-xl opacity-80 transform rotate-12 group-hover:rotate-6 transition-transform duration-500"
            }
            
            // Icon with enhanced styling
            div {
                class: "relative w-14 h-14 rounded-xl flex items-center justify-center mb-5 bg-gradient-to-br from-amber-500/20 to-orange-600/20 border border-amber-500/30 shadow-lg group-hover:shadow-amber-500/20 transition-all duration-500",
                span {
                    class: "text-3xl transform group-hover:scale-110 transition-transform duration-300",
                    style: "background: linear-gradient(135deg, #fbbf24, #d97706); -webkit-background-clip: text; -webkit-text-fill-color: transparent;",
                    "{props.icon}"
                }
            }
            
            // Title with gradient effect
            h3 {
                class: "text-xl font-bold mb-3 text-transparent bg-clip-text bg-gradient-to-r from-amber-400 to-orange-500 group-hover:from-amber-300 group-hover:to-orange-400 transition-colors duration-300",
                "{props.title}"
            }
            
            // Description with improved readability
            p {
                class: "text-gray-300 leading-relaxed group-hover:text-gray-200 transition-colors duration-300",
                "{props.description}"
            }
            
            // Subtle indicator line
            div {
                class: "absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-amber-500 to-orange-600 transform scale-x-0 group-hover:scale-x-100 transition-transform duration-500 origin-left"
            }
        }
    }
}
