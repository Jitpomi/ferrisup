use dioxus::prelude::*;

fn main() {
    // Launch the web app
    dioxus_web::launch(App);
}

// Define the main app component
fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "container mx-auto p-4",
            h1 {
                class: "text-3xl font-bold mb-4",
                "Welcome to FerrisUp!"
            }
            p {
                class: "mb-4",
                "This is a Dioxus web application created with FerrisUp."
            }
            p {
                "Edit ", code { "src/main.rs" }, " to customize this application."
            }
            div {
                class: "mt-8 p-4 border rounded shadow-sm",
                h2 {
                    class: "text-xl font-semibold mb-2", 
                    "Getting Started"
                }
                ul {
                    class: "list-disc pl-5",
                    li { "Edit the UI in this file" }
                    li { "Add new components in separate files" }
                    li { "Connect to your backend API" }
                    li { "Style with Tailwind or custom CSS" }
                }
            }
        }
    })
}
