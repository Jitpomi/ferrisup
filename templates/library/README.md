# {{project_name}}

This template provides a foundation for creating a Rust library crate with a well-structured `lib.rs` file and testing infrastructure.

## Features

- Library crate structure with documentation
- Unit tests setup
- Benchmarking with Criterion
- Error handling with anyhow
- Ready for publishing to crates.io

## Getting Started

After generating your project with FerrisUp, follow these steps:

1. Navigate to your project directory:
   ```bash
   cd {{project_name}}
   ```

2. Run the tests:
   ```bash
   cargo test
   ```

3. Run benchmarks:
   ```bash
   cargo bench
   ```

4. Build the documentation:
   ```bash
   cargo doc --open
   ```

## Project Structure

- `src/lib.rs`: Main library file with documentation and tests
- `Cargo.toml`: Project configuration with development dependencies

## Customization

### Adding Modules

As your library grows, consider organizing it into modules:

```rust
// In src/lib.rs
pub mod utils;
pub mod models;

// Create src/utils.rs or src/utils/mod.rs
// Create src/models.rs or src/models/mod.rs
```

### Documentation

The template includes doc comments. Expand them to document your API:

```rust
/// Performs an important calculation.
///
/// # Examples
///
/// ```
/// let result = {{project_name}}::calculate(42);
/// assert_eq!(result, 84);
/// ```
///
/// # Errors
///
/// Returns an error if the input is invalid.
pub fn calculate(input: i32) -> Result<i32, Error> {
    // Implementation
}
```

### Publishing

Prepare your library for publishing:

1. Update `Cargo.toml` with metadata:
   ```toml
   [package]
   name = "{{project_name}}"
   version = "0.1.0"
   authors = ["Your Name <your.email@example.com>"]
   edition = "2021"
   description = "A brief description of your library"
   repository = "https://github.com/yourusername/{{project_name}}"
   license = "MIT OR Apache-2.0"
   keywords = ["keyword1", "keyword2"]
   categories = ["category1", "category2"]
   ```

2. Publish to crates.io:
   ```bash
   cargo publish
   ```

## Next Steps

- Add your library's core functionality to `src/lib.rs`
- Create additional modules as needed
- Write comprehensive tests and examples
- Set up CI/CD with GitHub Actions
- Add a README.md with usage examples

## Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book: Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Rust Documentation Guidelines](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)
- [Criterion Benchmarking](https://bheisler.github.io/criterion.rs/book/)
