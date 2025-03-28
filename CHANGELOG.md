# Changelog

All notable changes to FerrisUp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-03-28

### Fixed
- Fixed critical index out of bounds error in the interactive project creator when using the full-stack template
- Added safe handling of client frameworks when there are more apps than frameworks defined

## [0.1.1] - 2025-03-28

### Added
- CHANGELOG.md for tracking version history
- CONTRIBUTING.md with guidelines for contributors

### Changed
- Updated package metadata in Cargo.toml
- Transferred GitHub repository to Jitpomi organization
- Updated documentation with correct repository links

### Fixed
- Fixed compiler warnings in various files

## [0.1.0] - 2025-03-28

### Added
- Initial release of FerrisUp
- Multiple templates (minimal, full-stack, gen-ai, edge-app, embedded)
- Transform command to convert existing projects to template structures
- List command to display available templates
- Scale command for interactively scaling projects
- Preview command to preview templates without creating files
- Component command for managing project components
- Config command for managing configurations
- Workspace management for multi-crate projects
- Support for AI framework integration
- Support for edge computing projects
- Support for embedded systems development
- Non-interactive mode for automated testing and CI/CD pipelines

### Changed
- N/A (Initial release)

### Fixed
- N/A (Initial release)
