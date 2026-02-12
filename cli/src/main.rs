pub mod auth;
pub mod cli;
pub mod config;
pub mod installer;
pub mod lockfile;
pub mod logger;
pub mod registry;
pub mod xml_handler;

use clap::Parser;
use cli::{Cli, Commands};
use logger::Logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Allow users to override the registry URL via CLI flag.
    // We use unsafe here because set_var mutates global state, but it's fine since we're doing it once at startup.
    // If you're uncomfortable with this, feel free to refactor to pass the URL through the call stack instead.
    if let Some(url) = &cli.api_url {
        unsafe {
            std::env::set_var("MOSAIC_REGISTRY_URL", url);
        }
    }

    // Enable verbose logging if requested
    if cli.verbose {
        if std::env::var("RUST_LOG").is_err() {
            unsafe {
                std::env::set_var("RUST_LOG", "debug");
            }
        }
        env_logger::init();
        Logger::debug("Verbose logging enabled");
    }

    match &cli.command {
        Commands::Init => {
            Logger::banner();
            // Get the directory name as a fallback project name.
            // If the user is in /home/alice/my-project, we use "my-project".
            let current_dir = std::env::current_dir()?;
            let project_name = current_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("my-mosaic-project");

            Logger::info(format!(
                "Initializing project: {}...",
                Logger::highlight(project_name)
            ));
            let config = config::Config::default(project_name);
            config.save()?;
            Logger::success("Created mosaic.toml");
        }

        Commands::Install { package } => {
            // Two modes:
            // 1. Install a specific package: mosaic install logger@1.0.0
            // 2. Install all from mosaic.toml: mosaic install (no args)
            if let Some(query) = package {
                let (package_name, resolved_version) = installer::install_package(query).await?;

                // Update mosaic.toml with the newly installed package.
                // We wrap this in a try-load because users might not have a config yet (weird edge case).
                if let Ok(mut config) = config::Config::load() {
                    config.add_dependency(&package_name, &resolved_version);
                    config.save()?;
                    Logger::info(format!(
                        "Added {} to mosaic.toml",
                        Logger::brand_text(&package_name)
                    ));
                }
            } else {
                // No package specified—install everything from mosaic.toml
                installer::install_all().await?;
            }
        }

        Commands::Remove { package } => {
            installer::remove_package(package).await?;
        }

        Commands::List => {
            installer::list_packages().await?;
        }

        Commands::Update => {
            // Update is basically just reinstall everything.
            // Could be smarter about checking what's out of date, but this works for now.
            installer::update_all().await?;
        }

        Commands::Login => {
            Logger::banner();
            registry::login().await?;
        }

        Commands::Logout => {
            registry::logout().await?;
        }

        Commands::Signup => {
            Logger::banner();
            registry::signup().await?;
        }

        Commands::Publish { version } => {
            // If the user provides --version, use that. Otherwise let the registry module handle it.
            registry::publish(version.as_deref()).await?;
        }

        Commands::Search { query } => {
            registry::search(query.clone()).await?;
        }

        Commands::Info { package } => {
            registry::info(package).await?;
        }
    }

    Ok(())
}
