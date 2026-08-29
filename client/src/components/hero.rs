use crate::components::buttons::Button;
use dioxus::prelude::*;

const FERRISUP_LOGO: Asset = asset!("assets/ferrisup-logo.png");

#[component]
pub fn Hero() -> Element {
    rsx! {
        header {
            class: "px-6 py-24 text-center max-w-6xl mx-auto",
            img {
                class: "brand-logo mx-auto",
                src: FERRISUP_LOGO,
                alt: "FerrisUp",
                width: "320",
                height: "320",
            }
            p { class: "text-amber-400 font-semibold tracking-wide uppercase", "Rust project foundations" }
            h1 { class: "mt-4 text-5xl sm:text-7xl font-bold text-white", "Structure that keeps up." }
            p {
                class: "mt-6 text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed",
                "FerrisUp starts Rust projects from inspectable templates and helps them grow into clear Cargo workspaces. It gives developers and coding agents the same explicit package boundaries, framework choices, and build commands."
            }
            nav {
                class: "mt-10 flex flex-col sm:flex-row gap-4 justify-center",
                aria_label: "Primary links",
                Button { href: "https://crates.io/crates/ferrisup", target: "_blank", rel: "noopener noreferrer", "Install FerrisUp" }
                Button { variant: "outline", href: "https://github.com/Jitpomi/ferrisup", target: "_blank", rel: "noopener noreferrer", "Read the source" }
            }
            pre {
                class: "mt-14 max-w-3xl mx-auto p-6 rounded-xl bg-gray-950 border border-gray-800 text-left overflow-x-auto",
                code { class: "text-sm text-gray-200", "cargo install ferrisup\n\nferrisup new api --component-type server --framework axum --git\nferrisup preview --component-type server --framework axum\nferrisup transform --project ./my-project" }
            }
        }
    }
}
