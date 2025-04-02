use leptos::prelude::*;

/// A simple counter component.
///
/// You can use doc comments like this to document your component.
#[component]
pub fn SimpleCounter(
    /// The starting value for the counter
    initial_value: i32,
    /// The change that should be applied each time the button is clicked.
    #[prop(default = 1)]
    step: i32,
) -> impl IntoView {
    let (value, set_value) = signal(initial_value);

    view! {
        <div class="counter-card">
            <span>"Value: " {value} "!"</span>
            <div class="button-container">
                <button on:click=move |_| set_value.set(0)>"Clear"</button>
                <button on:click=move |_| *set_value.write() -= step>"-1"</button>
                <button on:click=move |_| set_value.update(|value| *value += step)>"+1"</button>
            </div>
        </div>
    }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <main>
            <h1>"Welcome to Leptos!"</h1>
            <SimpleCounter initial_value=0 step=1/>
        </main>
    }
}
