use dioxus::prelude::*;

fn main() {
    // Launch the web app
    dioxus_web::launch(App);
}

// Define the main app component
fn App() -> Element {
    let count = use_signal(|| 0);

    rsx! {
        div { class: "container mx-auto p-4",
            h1 { class: "text-3xl font-bold mb-4",
                "Welcome to your Dioxus App"
            }
            p { class: "mb-4",
                "This is a starter template for your Dioxus web application."
            }
            div { class: "flex items-center gap-4 mb-8",
                button {
                    class: "px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600",
                    onclick: move |_| count.set(*count + 1),
                    "Count: {count}"
                }
                button {
                    class: "px-4 py-2 bg-gray-300 rounded hover:bg-gray-400",
                    onclick: move |_| count.set(0),
                    "Reset"
                }
            }
            div { class: "border-t pt-4",
                h2 { class: "text-xl font-semibold mb-2",
                    "Getting Started"
                }
                ul { class: "list-disc pl-5",
                    li {
                        "Edit ",
                        code { "src/main.rs" },
                        " to customize this application"
                    }
                    li {
                        "Visit the ",
                        a {
                            class: "text-blue-500 hover:underline",
                            href: "https://dioxuslabs.com/docs/",
                            target: "_blank",
                            "Dioxus documentation"
                        },
                        " to learn more"
                    }
                }
            }
        }
    }
}
