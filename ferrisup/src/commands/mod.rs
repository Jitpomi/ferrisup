// Export commands modules
pub mod new;
pub mod list;
pub mod preview;
pub mod transform;
pub mod config;
pub mod workspace;
pub mod component;
pub mod dependency;
pub mod unused_features;
pub mod import_fixer;
pub mod test_mode;
// Removed reference to unused module

// Re-export the Commands enum for the CLI
use clap::{Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ComponentType {
    Client,
    Server,
    Shared,
    Edge,
    Serverless,
    DataScience,
    Embedded,
    Library,
    Minimal,
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ComponentType::Client => "client",
            ComponentType::Server => "server",
            ComponentType::Shared => "shared",
            ComponentType::Edge => "edge",
            ComponentType::Serverless => "serverless",
            ComponentType::DataScience => "data-science",
            ComponentType::Embedded => "embedded",
            ComponentType::Library => "library",
            ComponentType::Minimal => "minimal",
        };
        write!(f, "{}", s)
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Rust project with interactive configuration
    New {
        /// Component name (optional, will prompt if not provided)
        #[arg(required = false)]
        name: Option<String>,

        /// Component type to use (optional, will prompt if not provided)
        #[arg(short, long)]
        component_type: Option<ComponentType>,
        
        /// Framework to use for client, server, or embedded components
        #[arg(long)]
        framework: Option<String>,
        
        /// Cloud provider for serverless components
        #[arg(long)]
        provider: Option<String>,
        
        /// Application type for edge components
        #[arg(long)]
        application_type: Option<String>,

        /// Project type for framework-specific options (e.g., desktop, web, mobile for Dioxus)
        #[arg(short, long)]
        project_type: Option<String>,

        /// Initialize a git repository
        #[arg(short, long)]
        git: bool,

        /// Run cargo build after project creation
        #[arg(short, long)]
        build: bool,

        /// Skip interactive prompts (for automated testing)
        #[arg(long)]
        no_interactive: bool,
    },

    /// Transform an existing project with interactive configuration
    Transform {
        /// Path to the project to transform (optional, will prompt if not provided)
        #[arg(short, long)]
        project: Option<String>,

        /// Template to transform to (optional, will prompt if not provided)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// List available component types
    List,

    /// Preview a component type without creating files
    /// 
    /// NOTE: This command is a work in progress and has several limitations:
    /// - Not all template variables are properly replaced
    /// - Some framework-specific features may not be accurately displayed
    /// - File previews may not reflect actual generated content
    /// - Limited support for complex template combinations
    #[cfg(not(feature = "workspace_test"))]
    Preview {
        /// Component type to preview (optional, will prompt if not provided)
        #[arg(short, long)]
        component_type: Option<ComponentType>,
        
        /// Framework to use for client, server, or embedded components
        #[arg(long)]
        framework: Option<String>,
        
        /// Cloud provider for serverless components
        #[arg(long)]
        provider: Option<String>,
        
        /// Application type for edge components
        #[arg(long)]
        application_type: Option<String>,
    },

    /// Manage project components (add/remove/list) with consistent component types
    #[cfg(not(feature = "workspace_test"))]
    Component {
        /// Action to perform: add, remove, or list
        #[arg(short, long)]
        action: Option<String>,

        /// Component type: client, server, ferrisup_common, edge, data-science, embedded
        #[arg(short, long)]
        component_type: Option<ComponentType>,

        /// Path to the project (optional, will use current directory if not provided)
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Manage configurations (export/import)
    #[cfg(not(feature = "workspace_test"))]
    Config {
        /// Export the current configuration to a file
        #[arg(short, long)]
        export: bool,

        /// Import a configuration from a file
        #[arg(short, long)]
        import: Option<String>,

        /// Path to export/import configuration (optional)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Manage Cargo workspaces
    Workspace {
        /// Action to perform: init, add, remove, list, or optimize
        #[arg(short, long)]
        action: Option<String>,

        /// Path to the workspace (optional, will use current directory if not provided)
        #[arg(short, long)]
        path: Option<String>,
    },
    
    /// Manage project dependencies
    #[cfg(not(feature = "workspace_test"))]
    Dependency(dependency::DependencyArgs),

    /// Find unused features in Cargo dependencies
    #[cfg(not(feature = "workspace_test"))]
    UnusedFeatures {
        /// Path to the project (optional, will use current directory if not provided)
        #[arg(short, long)]
        path: Option<String>,
    },
}
