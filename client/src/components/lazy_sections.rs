use crate::components::cta::CallToAction;
use crate::components::features::Features;
use crate::components::footer::Footer;
use dioxus::prelude::*;

#[component]
pub fn LazySections() -> Element {
    rsx! {
        Features {}
        CallToAction {}
        Footer {}
    }
}
