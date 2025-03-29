use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod config;
pub mod templates;
mod utils;

#[derive(Parser)]
#[command(
    name = "ferrisup",
    author,
    version,
    about = "A versatile Rust project bootstrapping tool - start anywhere, scale anywhere",
    long_about = "FerrisUp is a powerful CLI tool for bootstrapping Rust projects with various templates ranging from minimal binaries to full-stack applications with AI, edge computing, and embedded systems support."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Rust project with interactive configuration
    New {
        /// Project name (optional, will prompt if not provided)
        #[arg(required = false)]
        name: Option<String>,

        /// Template to use (optional, will prompt if not provided)
        #[arg(short, long)]
        template: Option<String>,

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

    /// List available templates
    List,

    /// Interactively scale a project with custom components
    Scale,

    /// Preview a template without actually creating files
    #[cfg(not(feature = "workspace_test"))]
    Preview {
        /// Template to preview (optional, will prompt if not provided)
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Manage project components (add/remove)
    #[cfg(not(feature = "workspace_test"))]
    Component {
        /// Action to perform: add, remove, or list
        #[arg(short, long)]
        action: Option<String>,

        /// Component type: client, server, database, ai, edge, embedded, etc.
        #[arg(short, long)]
        component_type: Option<String>,

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
    Dependency(commands::dependency::DependencyArgs),
}

fn main() -> Result<()> {
    // Welcome message
    println!("{}", r#"
  _____                _     _   _       
 |  ___|__ _ __ _ __ (_)___| | | |_ __  
 | |_ / _ \ '__| '__| / __| | | | '_ \ 
 |  _|  __/ |  | |  | \__ \ |_| | |_) |
 |_|  \___|_|  |_|  |_|___/\___/| .__/ 
                               |_|    
 "#.green().bold());
    println!("{}", "FerrisUp - A tool for bootstrapping Rust projects".green().bold());
    println!("{}", "Start anywhere, scale anywhere\n".green());

    env_logger::init();
    let cli = Cli::parse();

    // Match the CLI command and execute
    match cli.command {
        Some(Commands::New { name, template, git, build, no_interactive }) => {
            match &name {
                Some(n) => println!(
                    "{} {} {} {}",
                    "Creating".green().bold(),
                    "new".green().bold(),
                    n.cyan().bold(),
                    "project".green().bold()
                ),
                None => println!(
                    "{} {} {}",
                    "Creating".green().bold(),
                    "new".green().bold(),
                    "project".green().bold()
                )
            }
            commands::new::execute(name.as_deref(), template.as_deref(), git, build, no_interactive)
        }
        Some(Commands::Transform { project, template }) => {
            match &project {
                Some(p) => println!(
                    "{} {}",
                    "Transforming".yellow().bold(),
                    p.cyan().bold()
                ),
                None => println!(
                    "{}",
                    "Starting interactive project transformation".yellow().bold()
                )
            }
            commands::transform::execute(project.as_deref(), template.as_deref())
        }
        Some(Commands::List) => {
            println!("{}", "Listing available templates".blue().bold());
            commands::list::execute()
        }
        Some(Commands::Scale) => {
            println!("{}", "Scaling project".green().bold());
            commands::scale::execute()
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(Commands::Preview { template }) => {
            println!("{}", "Previewing template".green().bold());
            commands::preview::execute(template.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(Commands::Component { action, component_type, project }) => {
            println!("{}", "Managing components".green().bold());
            commands::component::execute(action.as_deref(), component_type.as_deref(), project.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(Commands::Config { export, import, path }) => {
            println!("{}", "Managing configuration".green().bold());
            commands::config::execute(export, import.as_deref(), path.as_deref())
        }
        Some(Commands::Workspace { action, path }) => {
            println!("{}", "Managing Cargo workspace".green().bold());
            commands::workspace::execute(action.as_deref(), path.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(Commands::Dependency(args)) => {
            println!("{}", "Managing dependencies".green().bold());
            commands::dependency::execute(args)
        }
        None => {
            println!("{}", "No command specified, using interactive mode".yellow());
            // Just show help for now
            Cli::parse_from(["ferrisup", "--help"]);
            Ok(())
        }
    }
}
