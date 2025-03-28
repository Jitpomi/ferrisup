# Testing FerrisUp

This document describes the testing approach for FerrisUp, a Rust CLI tool for bootstrapping and managing Rust projects.

## Test Structure

FerrisUp uses both unit tests and integration tests to ensure code quality and reliability:

1. **Unit Tests** - Located within individual Rust files, testing specific functions and components
2. **Integration Tests** - Located in the `/tests` directory, testing complete commands and workflows

## Running Tests

To run all tests:

```bash
cargo test
```

To run only unit tests:

```bash
cargo test --lib
```

To run only integration tests:

```bash
cargo test --test '*'
```

To run a specific test, use the test name:

```bash
cargo test config
```

Some integration tests are marked with `#[ignore]` because they take longer to run or have special requirements. Run these with:

```bash
cargo test -- --ignored
```

## Test Coverage

We aim for high test coverage across all components:

- **Config Module**: Config creation, parsing, and serialization
- **Templates**: Template verification and content generation
- **Commands**: Each command functionality (new, transform, scale, etc.)
- **CLI Interface**: End-to-end testing of the command-line interface

## Adding New Tests

When adding new features to FerrisUp, please follow these guidelines for tests:

1. Add unit tests for new functions within the same file (`#[cfg(test)] mod tests { ... }`)
2. For new commands, add both unit tests and integration tests
3. Update existing tests if you modify interface behaviors
4. Use the `common` module for shared testing utilities

## Mocking

We use tempfiles and test fixtures to avoid affecting the actual system during testing:

- Temporary directories with `tempfile::tempdir()`
- Mock project structures created with testing utilities
- Config fixtures for various templates

## Continuous Integration

All tests should pass on CI before merging to main. Our CI pipeline runs:

- Tests on multiple platforms (Linux, macOS, Windows)
- Clippy for linting
- Rustfmt for code formatting
- Test coverage reporting

## Publishing Checklist

Before publishing to cargo, ensure:

1. All tests pass: `cargo test --all-features`
2. Doc tests pass: `cargo test --doc`
3. Linting passes: `cargo clippy`
4. The README includes all required sections
5. Cargo metadata is complete (description, authors, license, repository)
