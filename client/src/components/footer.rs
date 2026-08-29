use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "px-6 py-12 bg-gray-950",
            div { class: "max-w-6xl mx-auto flex flex-col md:flex-row gap-6 justify-between",
                div {
                    p { class: "font-semibold text-white", "FerrisUp" }
                    p { class: "mt-2 text-gray-400", "Inspectable Rust foundations for projects that need room to grow." }
                }
                nav { class: "flex flex-wrap gap-6 text-gray-300", aria_label: "Project links",
                    a { href: "https://github.com/Jitpomi/ferrisup", target: "_blank", rel: "noopener noreferrer", "Source" }
                    a { href: "https://github.com/Jitpomi/ferrisup/blob/main/ferrisup/README.md", target: "_blank", rel: "noopener noreferrer", "Documentation" }
                    a { href: "https://crates.io/crates/ferrisup", target: "_blank", rel: "noopener noreferrer", "Crates.io" }
                    a { href: "https://github.com/Jitpomi/ferrisup/issues", target: "_blank", rel: "noopener noreferrer", "Issues" }
                }
            }
        }
    }
}
