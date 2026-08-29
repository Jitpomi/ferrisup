use crate::components::buttons::Button;
use dioxus::prelude::*;

#[component]
pub fn CallToAction() -> Element {
    rsx! {
        section { class: "px-6 py-20 border-y border-gray-800 bg-gray-950",
            div { class: "max-w-5xl mx-auto grid lg:grid-cols-2 gap-12 items-center",
                div {
                    h2 { class: "text-4xl font-bold text-white", "Why this matters in an AI workflow" }
                    p { class: "mt-5 text-lg text-gray-300 leading-relaxed", "AI can write code quickly. FerrisUp addresses a different problem: making the repository shape explicit before that code accumulates. The result is less ambiguity, smaller architectural drift, and a baseline every contributor can inspect." }
                    p { class: "mt-4 text-gray-400", "FerrisUp is a structural tool, not an AI framework. You keep the source, manifests, tests, and responsibility for the result." }
                }
                div { class: "p-7 rounded-xl bg-gray-900 border border-gray-800",
                    h3 { class: "font-semibold text-white", "A practical workflow" }
                    ol { class: "mt-4 list-decimal list-inside space-y-3 text-gray-300",
                        li { "Choose a component and framework explicitly." }
                        li { "Generate and commit the baseline." }
                        li { "Give the agent the manifests and acceptance criteria." }
                        li { "Require formatting, linting, and tests." }
                        li { "Review security and deployment decisions." }
                    }
                    div { class: "mt-7", Button { href: "https://github.com/Jitpomi/ferrisup/blob/main/ferrisup/README.md", target: "_blank", rel: "noopener noreferrer", "Open the CLI guide" } }
                }
            }
        }
    }
}
