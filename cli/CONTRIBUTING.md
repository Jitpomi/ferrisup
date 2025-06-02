# Contributing to FerrisUp

Thank you for your interest in contributing to FerrisUp! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

By participating in this project, you agree to abide by the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## How to Contribute

### Reporting Bugs

Before submitting a bug report:
- Check the issue tracker to see if the bug has already been reported
- Ensure you're using the latest version of FerrisUp
- Collect information about your environment (OS, Rust version, etc.)

When submitting a bug report:
- Use a clear and descriptive title
- Describe the exact steps to reproduce the issue
- Explain the behavior you observed and what you expected to see
- Include relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are welcome! When submitting enhancement suggestions:
- Use a clear and descriptive title
- Provide a detailed description of the suggested enhancement
- Explain why this enhancement would be useful to FerrisUp users
- Include examples of how the enhancement would work if applicable

### Contributing Code

1. Fork the repository
2. Create a new branch for your feature or bugfix
3. Write code following Rust style guidelines
4. Add tests for your changes
5. Ensure all tests pass with `cargo test`
6. Submit a pull request

#### Pull Request Process

1. Update the README.md and documentation with details of your changes if applicable
2. Add your changes to the CHANGELOG.md under the "Unreleased" section
3. The PR should work on the main development branch
4. Maintainers will review your PR and may request changes
5. Once approved, maintainers will merge your PR

### Adding Templates

When adding new templates:
1. Follow the existing template structure
2. Ensure the template is fully tested
3. Document the template in the README.md
4. Add examples of how to use the template

## Development Setup

To set up your development environment:

```bash
# Clone the repository
git clone https://github.com/Jitpomi/ferrisup.git
cd ferrisup

# Install dependencies
cargo build

# Run tests
cargo test
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with verbose output
cargo test -- --nocapture
```

## Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` to format your code
- Use `clippy` to catch common mistakes

## License

By contributing to FerrisUp, you agree that your contributions will be licensed under the project's MIT license.
