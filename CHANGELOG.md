# Changelog

All notable changes to FerrisUp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.18] - 2025-06-01

### Key Milestones
- **Unified Component Creation**: Centralized component creation logic for consistent behavior
- **Standardized Dependency Management**: Single consistent approach across all commands
- **Improved User Experience**: Clearer menus, better prompts, and more intuitive workflows
- **Comprehensive Roadmap**: Detailed plan for both immediate and long-term development

### Added
- Comprehensive Future Work section in README with detailed roadmap:
  - Configuration system integration plans for future versions
  - Scaling and deployment enhancements
  - Database components and integration
  - Embedded systems and IoT enhancements based on user feedback
  - Network automation and mesh networking
  - Bare metal and self-hosted deployment
  - CI/CD and deployment automation
- Enhanced Features in Development section with clear priorities for version 1.0:
  - Core functionality improvements
  - Developer experience enhancements
  - Stability and testing priorities
- Example usage patterns for component command:
  ```bash
  # Add a component to a project
  ferrisup component --action add
  
  # List all components in a project
  ferrisup component --action list
  
  # Add a component to a specific project
  ferrisup component --action add --project /path/to/project
  ```

### Changed
- Refactored component command to properly delegate to transform command functions:
  - Workspace-aware component creation using the appropriate transform functions
  - Consistent component type selection based on project structure
  - Improved user experience with clearer prompts and messages
- Standardized dependency management across the codebase:
  - Unified dependency handling in all commands
  - Consistent approach to adding and managing dependencies
  - Improved documentation for dependency command
  - Example usage:
    ```bash
    # Add a dependency to a project
    ferrisup dependency add serde --features derive
    
    # Add multiple dependencies at once
    ferrisup dependency add tokio axum tower
    
    # Add a development dependency
    ferrisup dependency add --dev tracing
    
    # Check for unused features in dependencies
    ferrisup unused-features
    ```
- Updated CLI help texts and documentation for consistency across commands
- Improved component command documentation in README
- Documented config command functionality for future integration

### Fixed
- Fixed component selection flow to avoid showing unintended secondary menus
- Fixed type mismatch errors in component command arguments
- Fixed dependency command to ensure consistent behavior with other commands:
  - Corrected dependency command to use `--version` flag instead of incorrect `--vers` flag
  - Fixed `--dev` flag to properly add dependencies to dev-dependencies section
  - Enhanced dependency management to properly handle moving dependencies between regular and dev sections
  - Added `--force` flag to ensure proper overwriting when changing dependency types
- Removed legacy component creation code for better maintainability
- Fixed inconsistencies in component type handling between workspace and non-workspace projects

### Investigated
- Analyzed configuration system implementation and documented findings
- Determined that config command is functional but needs better integration
- Identified potential future benefits of the configuration system
- Prioritized config integration for future versions in the roadmap
- Current config command functionality:
  ```bash
  # Export current configuration to a file
  ferrisup config --export --path config.json
  
  # Import configuration from a file
  ferrisup config --import config.json
  
  # Interactive configuration management
  ferrisup config
  ```
- Future integration will enable persistent user preferences and template customization

## [0.1.17] - 2025-05-31

### Changed
- Updated Quick Start examples in README to better demonstrate component types and frameworks:
  - Added example for creating a minimal project with `--component-type minimal`
  - Added example for creating a Leptos client project with framework specification
  - Improved clarity of examples to match current command structure

### Removed
- Deleted unused example file: `examples/project_handlers_config.json`

## [0.1.16] - 2025-05-31

### Changed
- Updated the Features In Development section in README to reflect current roadmap:
  - Scale command for deployment
  - Database components
  - Machine learning components

## [0.1.15] - 2025-05-31

### Added
- Enhanced transform command with comprehensive workspace management:
  - Intelligent component type detection when converting projects to workspaces
  - Default component name suggestions based on component type
  - Comprehensive file migration when converting to workspaces
  - Automatic source file reference updating to match new package names
  - Helpful "Next Steps" guide after workspace transformations

### Changed
- Improved component creation in transform command:
  - Now directly uses the `new` command functionality for consistent component creation
  - Uses the same framework options as the `new` command
  - Creates components with proper project-prefixed package naming conventions
  - Maintains consistency between workspace and non-workspace component creation
- Enhanced metadata storage:
  - Changed metadata location from `.ferrisup.toml` to `.ferrisup/metadata.toml`
  - Improved path handling with absolute project root paths

### Fixed
- Fixed template handling in transform command:
  - Properly handles all component types and frameworks
  - Skips template.json files during template application
  - Uses library template for shared components
  - Properly handles component creation in both workspace and non-workspace projects
- Fixed package naming to use underscores instead of hyphens for valid Rust identifiers
- Fixed duplicate component name prompts in the transform command

## [0.1.14] - 2025-05-24

### Fixed
- Fixed README display issue on crates.io by updating image reference to use an absolute URL

## [0.1.13] - 2025-05-24

### Fixed
- Fixed README display issue on crates.io
- Properly configured banner image display for crates.io

## [0.1.9] - 2025-05-24

### Added
- Added project banner image to README.md
- Improved crates.io presentation with banner image

### Fixed
- Fixed duplicate metadata in Cargo.toml

## [0.1.8] - 2025-05-24

### Added
- New `unused-features` command to help optimize Rust projects by identifying and removing unused features in dependencies
  - Automatically installs the required `cargo-unused-features` tool if not present
  - Analyzes project dependencies for unused features
  - Provides clear, formatted output of unused features grouped by dependency
  - Generates specific recommendations for removing unused features from Cargo.toml

### Changed
- Improved template handling for better user experience:
  - Enhanced serverless templates with clearer next steps and better AWS Lambda environment variable documentation
  - Improved edge templates with more concise output and better post-creation guidance
  - Reduced verbosity in template output to focus on important information

### Fixed
- Fixed unused imports in Poem template to eliminate warnings
- Fixed directory structure in minimal template by moving main.rs from root to src directory
- Improved error handling in template processing

## [0.1.7] - 2025-03-28

### Added
- New `dependency` command for managing project dependencies with the following features:
  - Interactive dependency addition with smart feature suggestions
  - Dependency removal with interactive selection
  - Dependency updates (specific or all)
  - Dependency analysis with security audit integration
- Performance improvements in workspace management using proper TOML parsing

### Changed
- Refactored `update_workspace_members` function to use the TOML crate for more robust parsing
- Improved workspace member detection for better multi-crate project support

## [0.1.6] - 2025-03-28

### Added
- Comprehensive error handling throughout the codebase
- Improved robustness in file system operations

### Changed
- Replaced all unsafe `unwrap()` calls with proper error handling
- Enhanced test assertions with descriptive error messages
- Improved server error handling in generated templates

### Fixed
- Potential panics when dealing with invalid file paths
- Improved error propagation in workspace management functions

## [0.1.5] - 2025-03-28

### Added
- Enhanced template customization with more granular options
- Improved database migration support with better tool integration
- Added progress indicators for long-running operations
- Enhanced error messages with more descriptive text and resolution suggestions
- Added validation checks to ensure generated projects are always in a buildable state

### Changed
- Optimized project creation process for better performance
- Improved command documentation with examples and detailed explanations
- Enhanced help text for all commands

### Fixed
- Addressed all compiler warnings throughout the codebase
- Fixed remaining unwrap() calls to use proper error handling
- Refactored common code patterns into helper functions for better maintainability

## [0.1.4] - 2025-04-01

### Changed
- Improved error handling throughout the codebase by replacing `unwrap()` calls with proper error handling
- Modified `create_directory` function to accept a `Path` instead of a string to reduce potential panics
- Updated project name extraction to handle potential errors gracefully

### Fixed
- Fixed potential panics in file path handling by properly handling `Option` types
- Improved robustness when creating project directories and files
- Fixed "index out of bounds" panic in the interactive project creator
- Fixed unwrap() calls in component management functionality

## [0.1.3] - 2025-03-28

### Added
- Enhanced database configuration with support for multiple database types:
  - Primary databases (PostgreSQL, MySQL, SQLite, MongoDB, TypeDB, CockroachDB, TimescaleDB, ScyllaDB)
  - Cache databases (Redis, Memcached, Hazelcast, Aerospike, Ignite)
  - Vector databases (Pinecone, Qdrant, Milvus, Chroma, Weaviate, Vespa, Faiss, OpenSearch)
  - Graph databases (Neo4j, TypeDB, ArangoDB, JanusGraph, DGraph, TigerGraph, Amazon Neptune)

### Fixed
- Fixed remaining index out of bounds errors in client and server setup functions
- Removed unused imports and variables to eliminate compiler warnings

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
