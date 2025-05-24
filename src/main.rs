use anyhow::Result;
use clap::Parser;
use colored::*;

// Use the library modules instead of local definitions
use ferrisup::commands;

// Keep utils for any necessary utility functions
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
    command: Option<commands::Commands>,
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
        Some(commands::Commands::New { name, template, project_type, git, build, no_interactive }) => {
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
            commands::new::execute(name.as_deref(), template.as_deref(), git, build, no_interactive, project_type.as_deref())
        }
        Some(commands::Commands::Transform { project, template }) => {
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
        Some(commands::Commands::List) => {
            println!("{}", "Listing available templates".blue().bold());
            commands::list::execute()
        }
        Some(commands::Commands::Scale) => {
            println!("{}", "Scaling project".green().bold());
            commands::scale::execute()
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Preview { template }) => {
            println!("{}", "Previewing template".green().bold());
            commands::preview::execute(template.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Component { action, component_type, project }) => {
            println!("{}", "Managing components".green().bold());
            commands::component::execute(action.as_deref(), component_type.as_deref(), project.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Config { export, import, path }) => {
            println!("{}", "Managing configuration".green().bold());
            commands::config::execute(export, import.as_deref(), path.as_deref())
        }
        Some(commands::Commands::Workspace { action, path }) => {
            println!("{}", "Managing Cargo workspace".green().bold());
            commands::workspace::execute(action.as_deref(), path.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Dependency(args)) => {
            println!("{}", "Managing dependencies".green().bold());
            commands::dependency::execute(args)
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::UnusedFeatures { path }) => {
            println!("{}", "Finding unused features in dependencies".green().bold());
            commands::unused_features::execute(path.as_deref())
        }
        None => {
            println!("{}", "No command specified, using interactive mode".yellow());
            // Just show help for now
            Cli::parse_from(["ferrisup", "--help"]);
            Ok(())
        }
    }
}
