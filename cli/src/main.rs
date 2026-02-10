pub mod cli;
pub mod config;
pub mod github;
pub mod installer;
pub mod xml_handler;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => {
            let current_dir = std::env::current_dir()?;
            let project_name = current_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("my-mosaic-project");

            println!("Initializing mosaic project: {}...", project_name);
            let config = config::Config::default(project_name);
            config.save()?;
            println!("Created mosaic.toml");
        }
        Commands::Install { package } => {
            if let Some(query) = package {
                let package_name = installer::install_package(query).await?;

                // Try to load and update config
                if let Ok(mut config) = config::Config::load() {
                    config.add_dependency(&package_name, query);
                    config.save()?;
                    println!("Updated mosaic.toml");
                }
            } else {
                installer::install_all().await?;
            }
        }
        Commands::Remove { package } => {
            installer::remove_package(package).await?;
        }
    }

    Ok(())
}
