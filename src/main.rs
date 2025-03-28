use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;
mod config;
mod templates;
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
    command: Commands,
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
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            template,
            git,
            build,
        } => {
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
            commands::new::execute(name.as_deref(), template.as_deref(), git, build)
        }
        Commands::Transform { project, template } => {
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
        Commands::List => {
            println!("{}", "Listing available templates".blue().bold());
            commands::list::execute()
        }
        Commands::Scale => {
            println!("{}", "Starting interactive project builder".green().bold());
            commands::scale::execute()
        }
    }
}
