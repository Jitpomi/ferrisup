use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{A, Form, Route, Router, Routes, RoutingProgress},
    hooks::use_navigate,
    path,
};
use std::time::Duration;

/// Root component that sets up the router and meta tags
#[component]
pub fn App() -> impl IntoView {
    // Provides context for managing document title, meta tags, and other document head elements
    provide_meta_context();
    
    // Signal to track routing state for progress indicator
    let (is_routing, set_is_routing) = signal(false);

    view! {
        // Sets the document title
        <Title text="{{project_name}} - Leptos Router Example"/>

        // Configures meta tags for the document
        <Meta name="description" content="A router example built with Leptos"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router set_is_routing>
            // Shows a progress bar while async data are loading
            <div class="routing-progress">
                <RoutingProgress is_routing max_time=Duration::from_millis(250) />
            </div>
            
            <nav>
                // Using <A> component for client-side navigation with proper aria-current attribute
                <A href="/">"Home"</A>
                <A href="/about">"About"</A>
                <A href="/settings">"Settings"</A>
            </nav>
            
            <main>
                <Routes fallback=|| view! { <p>"Not Found"</p> }>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/about") view=AboutPage/>
                    <Route path=path!("/settings") view=Settings/>
                </Routes>
            </main>
        </Router>
    }
}

/// Home page component
#[component]
fn HomePage() -> impl IntoView {
    // Log when component renders
    #[cfg(debug_assertions)]
    {
        let _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();
    }
    
    on_cleanup(|| {
        // Cleanup code
    });
    
    view! {
        <div class="page">
            <h1>"Welcome to {{project_name}}"</h1>
            <p>"This is a simple example of routing with Leptos."</p>
            <p>"The router handles client-side navigation without full page reloads."</p>
            <ul>
                <li><A href="/about">"About"</A></li>
                <li><A href="/settings">"Settings"</A></li>
            </ul>
        </div>
    }
}

/// About page component
#[component]
fn AboutPage() -> impl IntoView {
    // Example of programmatic navigation
    let navigate = use_navigate();
    
    view! {
        <div class="page">
            <h1>"About"</h1>
            <p>"This is the about page for {{project_name}}."</p>
            <p>"You can navigate using links or programmatically:"</p>
            <button on:click=move |_| navigate("/", Default::default())>
                "Navigate Home Programmatically"
            </button>
            <p>
                <A href="/">"Home"</A>
            </p>
        </div>
    }
}

/// Settings page component
#[component]
fn Settings() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Settings"</h1>
            <Form action="">
                <fieldset>
                    <legend>"Name"</legend>
                    <input type="text" name="first_name" placeholder="First" />
                    <input type="text" name="last_name" placeholder="Last" />
                </fieldset>
                <input type="submit" />
                <p>
                    "This uses the " <code>"<Form/>"</code>
                    " component, which enhances forms by using client-side navigation for "
                    <code>"GET"</code> " requests, and client-side requests for " <code>"POST"</code>
                    " requests, without requiring a full page reload."
                </p>
            </Form>
            <p>
                <A href="/">"Home"</A>
            </p>
        </div>
    }
}

/// 404 Not Found page component
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"404 - Not Found"</h1>
            <p>"The page you requested could not be found."</p>
            <A href="/">"Return Home"</A>
        </div>
    }
}
