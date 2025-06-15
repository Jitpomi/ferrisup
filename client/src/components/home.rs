use dioxus::core_macro::{component, rsx};
use dioxus::prelude::*;
use crate::components::hero::Hero;
use crate::components::features::Features;
use crate::components::cta::CallToAction;
use crate::components::footer::Footer;

#[component]
pub fn HomePage() -> Element {
    rsx! {
        div {
            style: "background-color: #121212; color: white;",
            class: "min-h-screen",

            // Hero section
            Hero {}

            // Features section
            Features {}

            // CTA section
            CallToAction {}

            // Footer
            Footer {}
        }
    }
}