use leptos::*;
use leptos_meta::*;
use leptos_router::*;

/// Root component that sets up the router and meta tags
#[component]
pub fn App() -> impl IntoView {
    // Provides context for managing document title, meta tags, and other document head elements
    provide_meta_context();

    view! {
        // Sets the document title
        <Title text="Leptos Router Example"/>

        // Configures meta tags for the document
        <Meta name="description" content="A simple router example built with Leptos"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        <Router>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/about" view=AboutPage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Home page component
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Welcome to Leptos Router"</h1>
            <p>"This is a simple example of routing with Leptos."</p>
            <A href="/about">"About"</A>
        </div>
    }
}

/// About page component
#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"About"</h1>
            <p>"This is the about page."</p>
            <A href="/">"Home"</A>
        </div>
    }
}

/// 404 Not Found page component
#[component]
fn NotFound() -> impl IntoView {
    // Set a response status
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_axum::ResponseOptions>();
        resp.set_status(http::StatusCode::NOT_FOUND);
    }

    view! {
        <div class="page">
            <h1>"404 - Not Found"</h1>
            <p>"The page you requested could not be found."</p>
            <A href="/">"Return Home"</A>
        </div>
    }
}
