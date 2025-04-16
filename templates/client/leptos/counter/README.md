# Leptos Counter Example

This template provides a simple counter application built with Leptos, demonstrating reactive state management in a client-side rendered (CSR) Rust web application.

## Features

- Client-side rendering with Leptos
- Reactive state management
- Signal-based reactivity
- Component-based architecture
- WASM-based web application
- CSS styling

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd your-project-name
   ```

2. Install the required tools if you haven't already:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo install trunk
   ```

3. Start the development server:
   ```bash
   trunk serve --open
   ```

   This will build your application, start a development server, and open your application in a web browser.

## Project Structure

- `src/lib.rs`: Contains the `Counter` component with reactive state
- `src/main.rs`: Application entry point that mounts the `Counter` component
- `index.html`: HTML template for the application
- `style.css`: CSS styles for the application
- `Cargo.toml`: Project dependencies and configuration

## How It Works

The counter example demonstrates these key Leptos concepts:

1. **Signals**: Reactive state variables that automatically track dependencies
   ```rust
   let (count, set_count) = create_signal(0);
   ```

2. **Event Handling**: Responding to user interactions
   ```rust
   on:click=move |_| set_count.update(|n| *n += 1)
   ```

3. **Derived Computations**: Automatically updated values based on signals
   ```rust
   let double_count = move || count() * 2;
   ```

## Customization

### Adding More State

Extend the example by adding more signals:

```rust
let (name, set_name) = create_signal(String::from("Leptos"));

view! {
    <input 
        type="text"
        on:input=move |ev| {
            set_name(event_target_value(&ev));
        }
        prop:value=name
    />
    <p>"Hello, " {name}</p>
}
```

### Adding Effects

Use effects to perform side effects when reactive values change:

```rust
create_effect(move |_| {
    log::info!("Count changed to: {}", count());
});
```

## Next Steps

- Add more components to your application
- Implement form handling with Leptos
- Add routing with `leptos_router`
- Connect to a backend API
- Explore server-side rendering (SSR) with Leptos

## Resources

- [Leptos Documentation](https://leptos.dev/docs)
- [Leptos GitHub Repository](https://github.com/leptos-rs/leptos)
- [Leptos Examples](https://github.com/leptos-rs/leptos/tree/main/examples)
- [Trunk Documentation](https://trunkrs.dev/)
