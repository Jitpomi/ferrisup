use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/{{project_name}}.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=StaticSegment("about") view=AboutPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <div class="container">
            <h1>"Welcome to Leptos!"</h1>
            <button on:click=on_click>"Click Me: " {count}</button>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <div class="navigation">
                <a href="/about">"About"</a>
            </div>
        </div>
    }
}

/// Renders the about page.
#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="container">
            <h1>"About"</h1>
            <p>"This is a simple Leptos application with server-side rendering."</p>
            <p>"Here are some key features of Leptos:"</p>
            <ul>
                <li>"Fine-grained reactivity"</li>
                <li>"Server-side rendering"</li>
                <li>"Hydration"</li>
                <li>"Server functions"</li>
            </ul>
            
            <div class="navigation">
                <a href="/">"Back to Home"</a>
            </div>
        </div>
    }
}
