use dioxus::prelude::*;

#[component]
pub fn Features() -> Element {
    let items = [
        (
            "Repeatable foundations",
            "Generate ordinary Rust files from bundled templates that remain visible, reviewable, and version-controlled.",
        ),
        (
            "A path to workspaces",
            "Convert a single Cargo package into a workspace and add focused client, server, shared, edge, embedded, or data components.",
        ),
        (
            "Agent-readable structure",
            "Make package boundaries and build expectations concrete so AI coding tools spend less time guessing how the repository is organized.",
        ),
        (
            "Current ecosystem choices",
            "Start with maintained Rust editions and current framework lines while keeping every dependency in your own Cargo manifests.",
        ),
        (
            "Safer generation",
            "Reject unsafe names and existing destinations instead of silently overwriting files or accepting path-like input.",
        ),
        (
            "Useful maintenance tools",
            "Preview bundled templates and assist with components, workspaces, dependencies, configuration, and feature inspection.",
        ),
    ];

    rsx! {
        section { class: "px-6 py-20 max-w-6xl mx-auto", id: "value",
            h2 { class: "text-4xl font-bold text-white", "Built for codebases, not demos" }
            p { class: "mt-4 text-lg text-gray-400 max-w-3xl", "FerrisUp does not generate product logic or replace engineering judgment. It establishes a coherent starting structure that people and agents can extend using normal Rust tooling." }
            div { class: "mt-12 grid md:grid-cols-2 lg:grid-cols-3 gap-6",
                for (title, description) in items {
                    article { class: "p-7 rounded-xl bg-gray-900 border border-gray-800",
                        h3 { class: "text-xl font-semibold text-amber-400", "{title}" }
                        p { class: "mt-3 text-gray-300 leading-relaxed", "{description}" }
                    }
                }
            }
        }
    }
}
