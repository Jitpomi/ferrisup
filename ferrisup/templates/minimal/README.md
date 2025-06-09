# {{project_name}}

This template provides a minimal starting point for a Rust project with a simple `main.rs` file and no external dependencies.

## Features

- Clean, minimal structure
- No external dependencies
- Simple "Hello, World!" example
- Ready for customization

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd {{project_name}}
   ```

2. Run the program:
   ```bash
   cargo run
   ```

3. Build for release:
   ```bash
   cargo build --release
   ```

## Project Structure

- `src/main.rs`: Main application entry point
- `Cargo.toml`: Project configuration (initially with no dependencies)

## Customization

### Adding Dependencies

Edit the `Cargo.toml` file to add dependencies:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
```

### Expanding the Project

As your project grows, consider adding:

1. A `lib.rs` file for shared functionality:
   ```bash
   touch src/lib.rs
   ```

2. Module files in the `src` directory:
   ```bash
   mkdir -p src/utils
   touch src/utils/mod.rs
   ```

3. Tests in a separate directory:
   ```bash
   mkdir -p tests
   touch tests/integration_tests.rs
   ```

## Next Steps

- Add your own code to `src/main.rs`
- Add dependencies as needed in `Cargo.toml`
- Set up a Git repository:
  ```bash
  git init
  git add .
  git commit -m "Initial commit"
  ```
- Consider adding a `.gitignore` file for Rust projects

## Resources

- [The Rust Programming Language Book](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Rust Standard Library Documentation](https://doc.rust-lang.org/std/)
