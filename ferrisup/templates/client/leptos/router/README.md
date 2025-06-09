# Leptos Router Application

This template provides a foundation for building multi-page web applications with client-side navigation using Leptos and Leptos Router.

## Features

- Client-side routing with Leptos Router
- Multiple page components
- Navigation between pages
- URL parameter handling
- Nested routes
- Meta tags management
- CSS styling

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd your-project-name
   ```

2. Install the WebAssembly target if you haven't already:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. Install Trunk for serving your application:
   ```bash
   cargo install trunk
   ```

4. Start the development server:
   ```bash
   trunk serve --open
   ```

5. Your application will open in your default web browser at `http://localhost:8080`

## Project Structure

- `src/main.rs`: Application entry point
- `src/lib.rs`: Main application component and routing setup
- `index.html`: HTML template
- `style.css`: CSS styling

## Routing

The template includes several example routes:

```rust
<Router>
    <Routes>
        <Route path="/" view=HomePage/>
        <Route path="/about" view=AboutPage/>
        <Route path="/users" view=Users>
            <Route path=":id" view=UserProfile/>
        </Route>
        <Route path="/*any" view=NotFound/>
    </Routes>
</Router>
```

## Page Components

Each page is implemented as a separate component:

```rust
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Home Page"</h1>
            <p>"Welcome to the Leptos Router example!"</p>
            <nav>
                <A href="/about">"Go to About"</A>
                <A href="/users">"View Users"</A>
            </nav>
        </div>
    }
}
```

## URL Parameters

The router can extract parameters from URLs:

```rust
#[component]
fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());
    
    view! {
        <div class="user-profile">
            <h2>"User Profile"</h2>
            <p>"User ID: " {id}</p>
        </div>
    }
}
```

## Meta Tags

The template uses Leptos Meta for managing document metadata:

```rust
#[component]
fn App() -> impl IntoView {
    provide_meta_context();
    
    view! {
        <Html lang="en"/>
        <Title text="Leptos Router App"/>
        <Meta name="description" content="A multi-page application with Leptos Router"/>
        // Your router and components
    }
}
```

## Customization

### Adding New Routes

To add a new route:

1. Create a new component for your page
2. Add a new `Route` element to the `Routes` component
3. Link to your new route using the `A` component

Example:
```rust
#[component]
fn SettingsPage() -> impl IntoView {
    view! {
        <div class="page">
            <h1>"Settings"</h1>
            <p>"Configure your application settings here."</p>
        </div>
    }
}

// In your Router
<Route path="/settings" view=SettingsPage/>

// Link to it
<A href="/settings">"Settings"</A>
```

### Nested Routes

For more complex applications, you can nest routes:

```rust
<Route path="/dashboard" view=Dashboard>
    <Route path="/" view=DashboardHome/>
    <Route path="analytics" view=Analytics/>
    <Route path="settings" view=Settings/>
</Route>
```

## Next Steps

- Add authentication and protected routes
- Implement data fetching from an API
- Add state management for your application
- Enhance styling with a CSS framework
- Add animations for route transitions

## Resources

- [Leptos Documentation](https://leptos.dev/)
- [Leptos Router Documentation](https://docs.rs/leptos_router/latest/leptos_router/)
- [Leptos Meta Documentation](https://docs.rs/leptos_meta/latest/leptos_meta/)
- [Trunk Documentation](https://trunkrs.dev/)
- [WebAssembly Documentation](https://webassembly.org/)
