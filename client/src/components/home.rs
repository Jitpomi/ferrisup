use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use crate::components::hero::Hero;
use crate::components::lazy_sections::LazySections;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            style: "background-color: #121212; color: white;",
            class: "min-h-screen",

            // Hero section - loads immediately for FCP
            Hero {}

            // Lazy-loaded sections for better performance
            LazySections {}
        }
    }
}