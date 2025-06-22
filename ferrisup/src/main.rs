use anyhow::Result;
use clap::Parser;
use colored::Colorize;

// Use the library modules instead of local definitions
use ferrisup::commands;

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
    // High-quality ASCII art of Ferris (Rust mascot)
    // Convert img.png to ASCII art using image-to-ascii library

    println!("{}", r#"                                                 
                 ######                           
              ##########                          
            #########  ##                         
           ######### ####                         
           ##############                         
          @##############                         
           #############                          
          ###########                             
         ########## ###########                   
         #####  ##################                
         ###########################              
         ############ ##############              
           ################## #######             
            #########################             
            ##########   @###########             
          ##############  ###############         
        ##################################        
       ###################################        
       ###@#####  #########################       
           ###       #@#####@##############       
           ###              ##############        
                              ##########          
                              ########            

        ════════ FERRISUP CLI ════════  
        Start anywhere, scale anywhere  
        ═══════════════════════════════

"#.bright_green().bold());

    env_logger::init();
    let cli = Cli::parse();

    // Match the CLI command and execute
    match cli.command {
        Some(commands::Commands::New { name, component_type, framework, provider, application_type, project_type, git, build, no_interactive }) => {
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
            
            // Convert ComponentType to &str safely
            let component_type_str = component_type.as_ref().map(|ct| ct.to_string());
            let component_type_ref = component_type_str.as_deref();
            
            commands::new::execute(
                name.as_deref(), 
                component_type_ref, 
                framework.as_deref(), 
                provider.as_deref(), 
                application_type.as_deref(), 
                git, 
                build, 
                no_interactive, 
                project_type.as_deref()
            )
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
            println!("{}", "Listing available component types".blue().bold());
            commands::list::execute()
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Preview { component_type, framework, provider, application_type }) => {
            println!("{}", "Previewing component type".green().bold());
            // Convert ComponentType to &str for the preview command
            let component_type_str = component_type.map(|ct| ct.to_string());
            commands::preview::execute(component_type_str.as_deref(), framework.as_deref(), provider.as_deref(), application_type.as_deref())
        }
        #[cfg(not(feature = "workspace_test"))]
        Some(commands::Commands::Component { action, component_type, project }) => {
            println!("{}", "Managing components".green().bold());
            
            // Convert ComponentType to &str safely
            let component_type_str = component_type.as_ref().map(|ct| ct.to_string());
            let component_type_ref = component_type_str.as_deref();
            
            commands::component::execute(
                action.as_deref(), 
                component_type_ref, 
                project.as_deref()
            )
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
