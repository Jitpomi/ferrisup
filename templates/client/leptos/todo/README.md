# Leptos TodoMVC Example

This is a TodoMVC implementation using the Leptos framework. It demonstrates how to build a reactive web application with Rust and WebAssembly.

## Features

- Add, edit, and delete todos
- Mark todos as completed
- Filter todos (All, Active, Completed)
- Persist todos in local storage
- Clear completed todos
- Toggle all todos at once

## Getting Started

### Prerequisites

- Rust and Cargo
- [Trunk](https://trunkrs.dev/) for building and bundling

### Installation

Build and serve the application with Trunk:

```bash
trunk serve --open
```

This will open the application in your default web browser.

## Project Structure

- `src/lib.rs` - Main application logic and components
- `src/main.rs` - Entry point for the application
- `index.html` - HTML template
- `style.css` - TodoMVC CSS styles

## How It Works

The application uses Leptos signals for reactive state management. The main components are:

- `App` - The main application component
- `Todo` - Component for individual todo items
- `Todos` - Data structure for managing a collection of todos

Todos are persisted in the browser's local storage, so they will be available even after refreshing the page.

## Learn More

- [Leptos Documentation](https://leptos.dev/)
- [TodoMVC](https://todomvc.com/)
