![FerrisUp - A powerful Rust project bootstrapping tool](https://raw.githubusercontent.com/Jitpomi/ferrisup/main/ferrisup/img.png)

# FerrisUp: Rust Project Bootstrapping Tool

[![Crates.io](https://img.shields.io/crates/v/ferrisup.svg)](https://crates.io/crates/ferrisup)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

> **Start Anywhere, Scale Anywhere**

FerrisUp is a versatile Rust project bootstrapping tool that enables developers to create, transform, and scale Rust projects with ease. Unlike other project generators, FerrisUp supports the entire project lifecycle from simple beginnings to complex architectures.

## 🚀 Quick Links

- [**Detailed Documentation**](./ferrisup/README.md) - Complete usage guide and examples
- [**Installation Guide**](#-installation) - How to install FerrisUp
- [**Changelog**](./ferrisup/CHANGELOG.md) - Recent updates and version history
- [**Contributing**](#-contributing) - How to contribute to FerrisUp
- [**License**](#-license) - License information

## 🌟 Key Features

- **Project Transformation** - Convert single-crate projects to workspaces as they grow
- **Component-Based Architecture** - Specialized components for different use cases
- **Domain-Specific Components** - Optimized components for web, data science, embedded, and more
- **Smart Dependency Management** - Interactive dependency handling with feature selection
- **Framework Support** - Direct support for popular Rust frameworks per component type.
- **Cloud Provider Integration** - Optimized configurations for major cloud providers

## 🔍 What Makes FerrisUp Different?

Unlike traditional template generators like cargo-generate, FerrisUp focuses on project evolution. Start with a simple project and transform it as your needs grow, without having to recreate your project structure from scratch.

## 📦 Workspace Structure

This repository is organized as a Rust workspace with the following components:

- [`ferrisup`](./ferrisup/) - The main CLI tool and all its functionality

## 💻 Installation

```bash
# Install from crates.io
cargo install ferrisup

# Or install from source
git clone https://github.com/Jitpomi/ferrisup.git
cd ferrisup
cargo install --path ./ferrisup
```

## 🚀 Quick Start

```bash
# Create a new project (interactive mode)
ferrisup new

# Create a specific type of project
ferrisup new my_app --component-type server --framework axum

# Transform an existing project
ferrisup transform
```

For complete documentation and examples, see the [detailed README](./ferrisup/README.md).

## 🧪 Development

To build all components in the workspace:

```bash
cargo build
```

To run tests for all components:

```bash
cargo test
```

To add a new component to the workspace:

```bash
ferrisup transform
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

---

<p align="center">Built with ❤️ by <a href="https://github.com/Jitpomi">Jitpomi</a></p>
