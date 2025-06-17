

# ferrisup_common

Shared utilities and common functionality for the FerrisUp project. This crate provides core functionality used by the main FerrisUp CLI application.

## Purpose

The `ferrisup_common` crate serves as a shared library for the FerrisUp project, containing reusable utilities and functions that are used across different components of the project. By centralizing these common functions, we ensure consistency and reduce code duplication.

## Core Utilities

### File System Operations (`fs` module)
- `create_directory`: Creates directories with proper error handling
- `copy_directory`: Recursively copies directories with platform-specific handling
- `visit_dirs`: Traverses directories recursively
- Cross-platform file permission handling with conditional compilation for Unix/Windows

### Cargo Integration (`cargo` module)
- `read_cargo_toml`: Parses Cargo.toml files into structured data
- `write_cargo_toml_content`: Writes modified Cargo.toml content back to disk
- `update_workspace_members`: Manages workspace member entries in Cargo.toml
- `update_cargo_with_dependencies`: Adds or updates dependencies in Cargo.toml files

### String Utilities
- `to_pascal_case`: Converts strings like "hello_world" to "HelloWorld"
- `to_snake_case`: Converts strings like "HelloWorld" to "hello_world"

## Cross-Platform Support

This crate is designed to work on both Unix-based systems and Windows, using Rust's conditional compilation features (`#[cfg(unix)]`) to handle platform-specific code, particularly for file permissions.

## Usage

This crate is primarily used as an internal dependency for the FerrisUp CLI application. If you're developing or extending FerrisUp, you can use these utilities in your code:

```rust
// File system operations
use ferrisup_common::fs::create_directory;
use ferrisup_common::fs::copy_directory;

// Cargo.toml manipulation
use ferrisup_common::cargo::read_cargo_toml;
use ferrisup_common::cargo::update_workspace_members;

// String utilities
use ferrisup_common::to_pascal_case;
use ferrisup_common::to_snake_case;
```

## Project Structure

When making changes to the `ferrisup_common` crate, ensure that:

1. All functionality remains cross-platform compatible
2. Functions are properly documented with examples
3. Error handling is consistent and informative
4. Changes are reflected in the main FerrisUp application where necessary

## Conclusion

The `ferrisup_common` crate provides essential utilities and functionality for the FerrisUp project, ensuring consistency and reducing code duplication across the project. Its cross-platform compatibility and robust error handling make it a reliable dependency for the FerrisUp CLI application.
