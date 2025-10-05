use dioxus::prelude::*;
use crate::components::features::Features;
use crate::components::community::Community;
use crate::components::cta::CallToAction;
use crate::components::footer::Footer;

#[component]
pub fn LazySections() -> Element {
    let mut is_loaded = use_signal(|| false);
    
    // Use intersection observer to lazy load sections
    use_effect(move || {
        // Simulate intersection observer - load after a short delay
        let future = async move {
            gloo_timers::future::TimeoutFuture::new(100).await;
            is_loaded.set(true);
        };
        spawn(future);
    });
    
    rsx! {
        if is_loaded() {
            // Features section
            Features {}
            
            // Community section with LinkedIn embed
            Community {}
            
            // CTA section  
            CallToAction {}
            
            // Footer
            Footer {}
        } else {
            // Placeholder for lazy sections
            div {
                class: "min-h-screen bg-gray-900",
                style: "height: 300vh;" // Increased height to account for community section
            }
        }
    }
}
